#!/usr/bin/env python3
"""Bennett Valley dev server.

Serves static/ on http://localhost:8080 AND proxies /api/chat → Claude Haiku
and persists per-NPC conversation memory in memory/<npc_id>.txt.

Usage:
    ANTHROPIC_API_KEY=sk-ant-... python3 proxy.py

Replaces:  python3 -m http.server 8080  (from the static/ directory)
"""
import json
import os
import subprocess
import urllib.request
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import urlparse, parse_qs, unquote_plus

def _load_api_key() -> str:
    # 1. Prefer environment variable if explicitly set
    if key := os.environ.get("ANTHROPIC_API_KEY", ""):
        return key
    # 2. Fall back to macOS keychain
    try:
        result = subprocess.run(
            ["security", "find-generic-password",
             "-a", "bennett_valley", "-s", "anthropic_api_key", "-w"],
            capture_output=True, text=True, check=True,
        )
        return result.stdout.strip()
    except Exception:
        return ""

ANTHROPIC_API_KEY = _load_api_key()

# ── Multiplayer rooms (in-memory) ─────────────────────────────────────────
import random, string, threading, time as _time

MP_ROOMS = {}  # room_code -> { "state": str, "inputs": [], "created": float, "players": int }
MP_LOCK = threading.Lock()

def _mp_generate_code():
    return ''.join(random.choices(string.ascii_uppercase, k=4))

def _mp_cleanup_old():
    """Remove rooms older than 1 hour."""
    now = _time.time()
    with MP_LOCK:
        stale = [k for k, v in MP_ROOMS.items() if now - v["created"] > 3600]
        for k in stale:
            del MP_ROOMS[k]
BASE_DIR   = os.path.dirname(os.path.abspath(__file__))
STATIC_DIR = os.path.join(BASE_DIR, "static")
MEMORY_DIR = os.path.join(BASE_DIR, "memory")
MODEL      = "claude-haiku-4-5-20251001"
MAX_MEMORY_LINES = 4   # keep the last N NPC lines per character

# NPCs who know about Victor's backstory and may hint at it naturally.
VICTOR_GOSSIPS = {
    "Nora", "Otto", "Lily", "Cal", "Rue",
}
VICTOR_GOSSIP_NOTE = (
    " You know that Victor, the rival farmer, had been trying to buy the old farm for 18 years "
    "before the player outbid him with lottery money. It's a sore subject for him, but you feel "
    "some sympathy — he's not a villain, just someone who had a dream taken away. If it comes up "
    "naturally, you might hint at this, but don't force it."
)

# ── Lottery fame system ────────────────────────────────────────────────────────
# NPCs who are visibly impressed / excited by the lottery story.
LOTTERY_IMPRESSED = {"Tess", "Finn", "Vera", "Sage", "Pip", "Faye", "Bea", "Sol"}
# NPCs who are skeptical or slightly resentful of outsider money.
LOTTERY_RESENTFUL = {"Bram", "Rin", "Cass", "Dex", "Kit"}
# Practical / rural types who heard the gossip but don't especially care.
LOTTERY_INDIFFERENT = {"Doc", "Ward", "Tom", "Rex", "Hank"}
# Everyone else falls into the default "curious townsfolk" bucket.

_LOTTERY_BASE = (
    "The player recently won the lottery and used the winnings to buy the old Weatherfield farm "
    "at the edge of town. "
)


def _lottery_fame_ctx(npc: str, hearts: int) -> str:
    """Return lottery-fame context to inject into the NPC's system prompt.

    The context fades as friendship deepens — by 4 hearts the novelty has
    worn off and the player is just a neighbour.
    Victor is excluded: his resentment is covered by VICTOR_GOSSIP_NOTE.
    """
    if npc == "Victor" or hearts >= 4:
        return ""

    if npc in LOTTERY_RESENTFUL:
        if hearts == 0:
            return (
                _LOTTERY_BASE +
                "You're a bit wary — outsider money buying its way into a tight-knit community "
                "doesn't sit right with you. You haven't made up your mind about them yet. "
            )
        elif hearts <= 2:
            return (
                "You used to be suspicious of the lottery farmer, but you've warmed up a little. "
                "You still wonder sometimes if they truly belong here. "
            )
        else:  # hearts == 3
            return "You've mostly gotten past your initial doubts about the player. "

    elif npc in LOTTERY_IMPRESSED:
        if hearts == 0:
            return (
                _LOTTERY_BASE +
                "You think it's bold and a little romantic — using lottery winnings to chase a "
                "farming dream. You're genuinely curious about them. "
            )
        elif hearts <= 2:
            return "The player's backstory still fascinates you a little. "
        else:
            return ""

    elif npc in LOTTERY_INDIFFERENT:
        if hearts == 0:
            return (
                _LOTTERY_BASE +
                "You've heard the gossip but you're a practical person — what matters is "
                "whether they can pull their weight, not how they paid for the land. "
            )
        else:
            return ""

    else:  # default: curious townsfolk
        if hearts == 0:
            return (
                _LOTTERY_BASE +
                "Everyone in town is talking about it. You're curious but trying not to be "
                "too obvious about it. "
            )
        elif hearts == 1:
            return "You've gotten used to the player being around; the lottery talk has died down. "
        elif hearts <= 3:
            return ""
        else:
            return ""


def _sanitize(text: str) -> str:
    """Replace non-ASCII punctuation with ASCII equivalents for Macroquad's font."""
    return (text
        .replace("\u2026", "...")   # ellipsis
        .replace("\u2014", " - ")   # em dash
        .replace("\u2013", "-")     # en dash
        .replace("\u2018", "'")     # left single quote
        .replace("\u2019", "'")     # right single quote
        .replace("\u201c", '"')     # left double quote
        .replace("\u201d", '"')     # right double quote
        .replace("\n", " ")         # newlines → space
        .strip()
    )


def _read_memory(npc_id: str) -> str:
    path = os.path.join(MEMORY_DIR, f"{npc_id}.txt")
    if not os.path.exists(path):
        return ""
    with open(path, encoding="utf-8") as f:
        return f.read().strip()


def _append_memory(npc_id: str, text: str) -> None:
    os.makedirs(MEMORY_DIR, exist_ok=True)
    path = os.path.join(MEMORY_DIR, f"{npc_id}.txt")
    lines = []
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            lines = [l.rstrip() for l in f if l.strip()]
    lines.append(text.replace("\n", " ").strip())
    lines = lines[-MAX_MEMORY_LINES:]          # keep only the most recent
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=STATIC_DIR, **kwargs)

    def do_GET(self):
        parsed = urlparse(self.path)
        if parsed.path == "/api/chat":
            self._handle_chat(parse_qs(parsed.query))
        elif parsed.path == "/api/chat_reply":
            self._handle_chat_reply(parse_qs(parsed.query))
        elif parsed.path == "/api/victor_final":
            self._handle_victor_final(parse_qs(parsed.query))
        elif parsed.path == "/api/letter":
            self._handle_letter(parse_qs(parsed.query))
        elif parsed.path == "/api/memory":
            self._handle_memory_save(parse_qs(parsed.query))
        elif parsed.path == "/api/proposal":
            self._handle_proposal(parse_qs(parsed.query))
        elif parsed.path.endswith(".json") or parsed.path.endswith(".wasm"):
            # Serve static JSON/WASM with no-cache headers so config changes are picked up.
            # Temporarily intercept to inject the header, then delegate back.
            self._send_nocache_static(parsed.path)
        elif parsed.path == "/api/mp/state":
            self._handle_mp_state(parse_qs(parsed.query))
        elif parsed.path == "/api/mp/inputs":
            self._handle_mp_inputs(parse_qs(parsed.query))
        else:
            super().do_GET()

    def do_POST(self):
        parsed = urlparse(self.path)
        content_len = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_len).decode('utf-8') if content_len > 0 else ''
        if parsed.path == "/api/mp/create":
            self._handle_mp_create()
        elif parsed.path == "/api/mp/join":
            self._handle_mp_join(parse_qs(parsed.query))
        elif parsed.path == "/api/mp/sync":
            self._handle_mp_sync(parse_qs(parsed.query), body)
        elif parsed.path == "/api/mp/input":
            self._handle_mp_input(parse_qs(parsed.query), body)
        else:
            self.send_error(404)

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    # ── Multiplayer endpoints ────────────────────────────────────────────────

    def _handle_mp_create(self):
        _mp_cleanup_old()
        code = _mp_generate_code()
        with MP_LOCK:
            while code in MP_ROOMS:
                code = _mp_generate_code()
            MP_ROOMS[code] = {"state": "{}", "inputs": [], "created": _time.time(), "players": 1}
        self._send_json(json.dumps({"room": code}))

    def _handle_mp_join(self, params):
        code = params.get("room", [""])[0].upper()
        with MP_LOCK:
            if code in MP_ROOMS:
                MP_ROOMS[code]["players"] = 2
                self._send_json(json.dumps({"ok": True, "room": code}))
            else:
                self._send_json(json.dumps({"ok": False, "error": "Room not found"}))

    def _handle_mp_sync(self, params, body):
        """Host pushes its state snapshot."""
        code = params.get("room", [""])[0].upper()
        with MP_LOCK:
            if code in MP_ROOMS:
                MP_ROOMS[code]["state"] = body
                self._send_text("ok")
            else:
                self.send_error(404)

    def _handle_mp_state(self, params):
        """Guest polls for latest state."""
        code = params.get("room", [""])[0].upper()
        with MP_LOCK:
            if code in MP_ROOMS:
                self._send_json(MP_ROOMS[code]["state"])
            else:
                self._send_json('{"error":"no room"}')

    def _handle_mp_input(self, params, body):
        """Guest pushes an input event."""
        code = params.get("room", [""])[0].upper()
        with MP_LOCK:
            if code in MP_ROOMS:
                MP_ROOMS[code]["inputs"].append(body)
                self._send_text("ok")
            else:
                self.send_error(404)

    def _handle_mp_inputs(self, params):
        """Host polls for guest inputs and clears the queue."""
        code = params.get("room", [""])[0].upper()
        with MP_LOCK:
            if code in MP_ROOMS:
                inputs = MP_ROOMS[code]["inputs"]
                MP_ROOMS[code]["inputs"] = []
                self._send_json(json.dumps(inputs))
            else:
                self._send_json("[]")

    def _send_nocache_static(self, path: str) -> None:
        """Serve a static file with Cache-Control: no-store to prevent browser caching."""
        import os as _os
        file_path = _os.path.join(STATIC_DIR, path.lstrip("/"))
        if not _os.path.isfile(file_path):
            self.send_error(404)
            return
        with open(file_path, "rb") as f:
            data = f.read()
        import mimetypes as _mt
        ctype, _ = _mt.guess_type(file_path)
        ctype = ctype or "application/octet-stream"
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store, no-cache, must-revalidate")
        self.send_header("Pragma", "no-cache")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(data)

    # ── /api/chat ─────────────────────────────────────────────────────────────

    def _handle_chat(self, params):
        npc_id      = params.get("npc_id",      ["0"])[0]
        npc         = params.get("npc",          ["NPC"])[0]
        personality = params.get("personality",  ["friendly"])[0]
        friendship  = int(params.get("friendship", ["0"])[0])
        season      = params.get("season",       ["spring"])[0]
        day         = params.get("day",          ["1"])[0]
        charisma    = int(params.get("charisma", ["0"])[0])
        year        = int(params.get("year",     ["1"])[0])
        hearts      = friendship // 25

        # ── Morgan the journalist: special interview mode ─────────────────────
        if npc == "Morgan":
            self._handle_morgan_interview(params)
            return

        # Charisma shapes how the NPC perceives and responds to the player
        if charisma <= 2:
            charm_ctx = "The player is a newcomer who is still awkward in conversation."
        elif charisma <= 5:
            charm_ctx = "The player has a growing warmth that people appreciate."
        elif charisma <= 8:
            charm_ctx = "The player is well-liked and easy to talk to."
        else:
            charm_ctx = "The player is genuinely charming — people open up to them easily."

        # Inject persisted memory if any
        memory      = _read_memory(npc_id)
        memory_ctx  = (
            f" You remember saying to the player before: {memory}"
            if memory else ""
        )

        # Year context — Victor and gossip NPCs get tension if time is running out
        year_ctx = ""
        if year == 1:
            year_ctx = " The player has been in Bennett Valley less than a year."
        elif year <= 3:
            year_ctx = f" The player has been farming here for {year} year(s)."
        elif year == 4:
            year_ctx = (
                f" The player has been here {year} years now. Time is starting to feel heavy."
                " People wonder if old grudges will ever truly heal."
            )
        else:
            year_ctx = (
                f" The player has been here {year} years. Relationships that haven't grown"
                " by now feel increasingly unlikely to blossom."
            )

        gossip  = VICTOR_GOSSIP_NOTE if npc in VICTOR_GOSSIPS else ""
        lottery = _lottery_fame_ctx(npc, hearts)
        system = (
            f"You are {npc}, a villager in Bennett Valley, a cozy farming game. "
            f"Your personality: {personality}. "
            f"The player has {hearts} heart(s) of friendship with you. "
            f"{charm_ctx} "
            f"It is {season.capitalize()}, day {day}, Year {year}.{memory_ctx}{lottery}{year_ctx}{gossip} "
            "Respond with ONLY a valid JSON object (no markdown, no extra text) in this exact format:\n"
            '{"npc_line": "YOUR GREETING HERE", "options": ["OPTION 1", "OPTION 2", "OPTION 3"]}\n'
            "npc_line: your in-character greeting, 1-2 sentences, no quotation marks, no stage directions.\n"
            "options: 3 short things the player might naturally say back (5-10 words each, first-person, varied in tone)."
        )

        body = json.dumps({
            "model": MODEL,
            "max_tokens": 200,
            "system": system,
            "messages": [{"role": "user", "content": "Hello!"}],
        }).encode()

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "Content-Type": "application/json",
                "x-api-key": ANTHROPIC_API_KEY,
                "anthropic-version": "2023-06-01",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                raw = json.loads(resp.read())["content"][0]["text"].strip()
            # Strip markdown code fences if present (```json ... ```)
            if raw.startswith("```"):
                raw = raw.split("```", 2)[1]
                if raw.startswith("json"):
                    raw = raw[4:]
                raw = raw.strip()
            # Parse the structured JSON response
            parsed = json.loads(raw)
            npc_line = _sanitize(parsed.get("npc_line", "").strip())
            options  = parsed.get("options", [])
            # Clamp to exactly 3 options, pad with defaults if needed
            options = [_sanitize(str(o)) for o in options[:3]]
            while len(options) < 3:
                options.append("That's interesting.")
            result = json.dumps({"npc_line": npc_line, "options": options})
        except Exception as exc:
            print(f"[proxy] LLM error: {exc}")
            result = json.dumps({
                "npc_line": f"*{npc} nods quietly.*",
                "options": ["Good to see you.", "How's it going?", "Bye for now."],
            })

        self._send_json(result)

    # ── /api/chat_reply ───────────────────────────────────────────────────────

    def _handle_chat_reply(self, params):
        npc_id      = params.get("npc_id",      ["0"])[0]
        npc         = params.get("npc",          ["NPC"])[0]
        personality = params.get("personality",  ["friendly"])[0]
        friendship  = int(params.get("friendship", ["0"])[0])
        season      = params.get("season",       ["spring"])[0]
        day         = params.get("day",          ["1"])[0]
        charisma    = int(params.get("charisma", ["0"])[0])
        year        = int(params.get("year",     ["1"])[0])
        player_said = unquote_plus(params.get("player_said", [""])[0])
        hearts      = friendship // 25

        # Morgan's follow-up is a brief journalist sign-off
        if npc == "Morgan":
            system = (
                "You are Morgan, a journalist passing through Bennett Valley to write a human-interest "
                "piece on the lottery farmer. You are sharp, observant, and professionally warm. "
                f"The player just said: \"{player_said}\". "
                "Give ONE short in-character reaction — acknowledge what they said and hint at how "
                "you'll frame it in your article (1 sentence). No quotation marks. No stage directions."
            )
            body = json.dumps({
                "model": MODEL,
                "max_tokens": 80,
                "system": system,
                "messages": [{"role": "user", "content": player_said}],
            }).encode()
            req = urllib.request.Request(
                "https://api.anthropic.com/v1/messages",
                data=body,
                headers={
                    "Content-Type": "application/json",
                    "x-api-key": ANTHROPIC_API_KEY,
                    "anthropic-version": "2023-06-01",
                },
            )
            try:
                with urllib.request.urlopen(req, timeout=15) as resp:
                    data = json.loads(resp.read())
                    text = _sanitize(data["content"][0]["text"])
            except Exception as exc:
                print(f"[proxy] morgan reply LLM error: {exc}")
                text = "Interesting. That'll make a good quote."
            self._send_text(text)
            return

        married     = params.get("married", ["false"])[0].lower() == "true"
        gossip  = VICTOR_GOSSIP_NOTE if npc in VICTOR_GOSSIPS else ""
        lottery = _lottery_fame_ctx(npc, hearts)
        spouse_ctx = " You are married to the player and deeply in love. Respond with warmth and intimacy befitting a spouse." if married else ""
        system = (
            f"You are {npc}, a villager in Bennett Valley, a cozy farming game. "
            f"Your personality: {personality}. "
            f"The player has {hearts} heart(s) of friendship with you. "
            f"It is {season.capitalize()}, day {day}, Year {year}.{lottery}{gossip}{spouse_ctx} "
            "Give ONE short in-character reaction to what the player just said (1 sentence). "
            "No quotation marks. No stage directions. Just your words."
        )

        body = json.dumps({
            "model": MODEL,
            "max_tokens": 80,
            "system": system,
            "messages": [{"role": "user", "content": player_said}],
        }).encode()

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "Content-Type": "application/json",
                "x-api-key": ANTHROPIC_API_KEY,
                "anthropic-version": "2023-06-01",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                data = json.loads(resp.read())
                text = _sanitize(data["content"][0]["text"])
        except Exception as exc:
            print(f"[proxy] chat_reply LLM error: {exc}")
            text = "Hm, that's something to think about."

        self._send_text(text)

    # ── Morgan journalist interview ───────────────────────────────────────────

    def _handle_morgan_interview(self, params):
        """Generate Morgan's interview question + 3 attitude-tagged response options."""
        season      = params.get("season", ["spring"])[0]
        day         = params.get("day",    ["1"])[0]
        year        = int(params.get("year", ["1"])[0])
        friendship  = int(params.get("friendship", ["0"])[0])
        hearts      = friendship // 25

        if hearts == 0:
            familiarity = "This is the first time the player has spoken to you."
        elif hearts <= 2:
            familiarity = "You've spoken briefly before. You're warming up to them."
        else:
            familiarity = "You've interviewed them before. You're on friendly terms."

        system = (
            "You are Morgan, a sharp and observant journalist passing through Bennett Valley "
            "to write a human-interest piece on the lottery farmer who bought the old Weatherfield farm. "
            f"It is {season.capitalize()}, day {day}, Year {year}. {familiarity} "
            "Ask ONE probing but good-natured interview question about their life here "
            "(farming, the community, or what the lottery meant to them). Keep it to 1-2 sentences. "
            "Then provide exactly 3 response options the player might give, each 6-10 words, first-person. "
            "Option 1 must be humble/self-deprecating. "
            "Option 2 must be confident/proud. "
            "Option 3 must be deflecting/evasive. "
            "Respond with ONLY a valid JSON object (no markdown, no extra text):\n"
            '{"npc_line": "MORGAN\'S QUESTION", "options": ["[Humble] ...", "[Confident] ...", "[Deflecting] ..."]}'
        )

        body = json.dumps({
            "model": MODEL,
            "max_tokens": 220,
            "system": system,
            "messages": [{"role": "user", "content": "Ready for the interview."}],
        }).encode()

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "Content-Type": "application/json",
                "x-api-key": ANTHROPIC_API_KEY,
                "anthropic-version": "2023-06-01",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                raw = json.loads(resp.read())["content"][0]["text"].strip()
            if raw.startswith("```"):
                raw = raw.split("```", 2)[1]
                if raw.startswith("json"):
                    raw = raw[4:]
                raw = raw.strip()
            parsed   = json.loads(raw)
            npc_line = _sanitize(parsed.get("npc_line", "").strip())
            options  = [_sanitize(str(o)) for o in parsed.get("options", [])[:3]]
            while len(options) < 3:
                options.append(["[Humble] Just doing my best.", "[Confident] I knew it would work out.", "[Deflecting] Hard to say, really."][len(options)])
            result = json.dumps({"npc_line": npc_line, "options": options})
        except Exception as exc:
            print(f"[proxy] morgan interview LLM error: {exc}")
            result = json.dumps({
                "npc_line": "So — what made you trade city life for mud and seeds?",
                "options": [
                    "[Humble] I just needed a fresh start, honestly.",
                    "[Confident] I always knew farming was my calling.",
                    "[Deflecting] It's complicated. Next question?",
                ],
            })

        self._send_json(result)

    # ── /api/proposal ─────────────────────────────────────────────────────────

    def _handle_proposal(self, params):
        """Generate an NPC's marriage proposal reaction + 3 player response options."""
        npc         = params.get("npc",         ["someone"])[0]
        personality = params.get("personality", ["kind"])[0]
        friendship  = int(params.get("friendship", ["0"])[0])
        hearts      = friendship // 25
        season      = params.get("season", ["spring"])[0]
        day         = params.get("day",    ["1"])[0]
        year        = int(params.get("year", ["1"])[0])
        charisma    = int(params.get("charisma",  ["0"])[0])

        system = (
            f"You are {npc}, a {personality} resident of Bennett Valley. "
            f"It is {season.capitalize()}, day {day}, Year {year}. "
            f"The player has just offered you a pendant and is proposing marriage. "
            f"You have {hearts}/10 hearts of friendship with them. "
            f"Their charisma level is {charisma}/10. "
            "React with a heartfelt, in-character marriage proposal scene. "
            "Your line should be 2-3 sentences — romantic, warm, and true to your personality. "
            "Then provide exactly 3 first-person player responses (6-10 words each). "
            "Option 1 must be an enthusiastic acceptance: start with [Accept]. "
            "Option 2 must be a tender but hesitant acceptance: start with [Accept]. "
            "Option 3 must be a gentle 'not yet': start with [NotYet]. "
            "Respond with ONLY a valid JSON object (no markdown, no extra text):\n"
            '{"npc_line": "NPC\'S PROPOSAL REACTION", "options": ["[Accept] ...", "[Accept] ...", "[NotYet] ..."]}'
        )

        body = json.dumps({
            "model": MODEL,
            "max_tokens": 250,
            "system": system,
            "messages": [{"role": "user", "content": "I offer you this pendant."}],
        }).encode()

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "Content-Type": "application/json",
                "x-api-key": ANTHROPIC_API_KEY,
                "anthropic-version": "2023-06-01",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                raw = json.loads(resp.read())["content"][0]["text"].strip()
            if raw.startswith("```"):
                raw = raw.split("```", 2)[1]
                if raw.startswith("json"):
                    raw = raw[4:]
                raw = raw.strip()
            parsed   = json.loads(raw)
            npc_line = _sanitize(parsed.get("npc_line", "").strip())
            options  = [_sanitize(str(o)) for o in parsed.get("options", [])[:3]]
            while len(options) < 3:
                fallbacks = [
                    "[Accept] Yes, with all my heart!",
                    "[Accept] I thought you'd never ask...",
                    "[NotYet] I need a little more time.",
                ]
                options.append(fallbacks[len(options)])
            result = json.dumps({"npc_line": npc_line, "options": options})
        except Exception as exc:
            print(f"[proxy] proposal LLM error: {exc}")
            result = json.dumps({
                "npc_line": f"I... this pendant... {npc} looks at you with shining eyes. I've been hoping you'd ask.",
                "options": [
                    "[Accept] Yes, with all my heart!",
                    "[Accept] I thought you'd never ask...",
                    "[NotYet] I need a little more time.",
                ],
            })

        self._send_json(result)

    # ── /api/letter ───────────────────────────────────────────────────────────

    def _handle_letter(self, params):
        friend      = params.get("friend",      ["Friend"])[0]
        personality = params.get("personality", ["friendly"])[0].replace("+", " ")
        season      = params.get("season",      ["spring"])[0]
        day         = int(params.get("day",     ["1"])[0])
        gold        = params.get("gold",        ["0"])[0]

        system = (
            f"You are {friend}, an old friend of the player who grew up with them before they "
            f"left to run a farm in Bennett Valley. Your personality is {personality}. "
            f"The player is now in {season.capitalize()}, day {day} of their farm life, "
            f"and has earned {gold} gold so far. "
            "Write a short letter to your old friend — mention something small from your own life, "
            "ask a curious question about the farm or valley, and end warmly. "
            "2-3 sentences only. No salutation, no sign-off. Just the body of the letter."
        )

        body = json.dumps({
            "model": MODEL,
            "max_tokens": 140,
            "system": system,
            "messages": [{"role": "user", "content": "Write the letter."}],
        }).encode()

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "Content-Type": "application/json",
                "x-api-key": ANTHROPIC_API_KEY,
                "anthropic-version": "2023-06-01",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                data = json.loads(resp.read())
                text = data["content"][0]["text"].strip()
        except Exception as exc:
            print(f"[proxy] letter LLM error: {exc}")
            text = "Hope the farm is treating you well — miss you around here."

        self._send_text(text)

    # ── /api/victor_final ─────────────────────────────────────────────────────

    def _handle_victor_final(self, params):
        season = params.get("season", ["spring"])[0]
        day    = params.get("day",    ["1"])[0]

        system = (
            "You are Victor, a rival farmer in Bennett Valley who arrived as a resentful, "
            "competitive newcomer determined to outshine everyone. Over many seasons, the player "
            "has shown patience, kindness, and genuine interest in your story. "
            f"It is {season.capitalize()}, day {day}. "
            "This is the moment you finally open up and acknowledge the player's friendship. "
            "Give a heartfelt, in-character final speech: admit you were wrong to be so guarded, "
            "thank the player sincerely, and say something hopeful about the future of the valley. "
            "2-3 sentences. No quotation marks. No stage directions. Just your words."
        )

        body = json.dumps({
            "model": MODEL,
            "max_tokens": 160,
            "system": system,
            "messages": [{"role": "user", "content": "Victor, I just wanted to say... I'm glad we became friends."}],
        }).encode()

        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "Content-Type": "application/json",
                "x-api-key": ANTHROPIC_API_KEY,
                "anthropic-version": "2023-06-01",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                data = json.loads(resp.read())
                text = data["content"][0]["text"].strip()
        except Exception as exc:
            print(f"[proxy] victor_final LLM error: {exc}")
            text = "...I suppose I owe you an apology. You're not so bad after all."

        self._send_text(text)

    # ── /api/memory ───────────────────────────────────────────────────────────

    def _handle_memory_save(self, params):
        npc_id = params.get("npc_id", ["0"])[0]
        text   = unquote_plus(params.get("text",   [""])[0])
        if text:
            _append_memory(npc_id, text)
        self._send_text("ok")

    # ── helpers ───────────────────────────────────────────────────────────────

    def _send_text(self, text: str) -> None:
        encoded = text.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(encoded)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(encoded)

    def _send_json(self, text: str) -> None:
        encoded = text.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(encoded)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(encoded)

    def log_message(self, fmt, *args):  # silence request logs
        pass


if __name__ == "__main__":
    if not ANTHROPIC_API_KEY:
        print("WARNING: ANTHROPIC_API_KEY not set — NPC responses will be fallback text.")
    os.makedirs(MEMORY_DIR, exist_ok=True)
    server = HTTPServer(("", 8080), Handler)
    print("Bennett Valley dev server → http://localhost:8080")
    print(f"Memory stored in: {MEMORY_DIR}/")
    server.serve_forever()
