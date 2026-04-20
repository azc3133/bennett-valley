#[derive(Debug, Clone)]
pub struct DialogueState {
    pub npc_name: String,
    pub lines: Vec<String>,
    pub current_line: usize,
}

impl DialogueState {
    pub fn new(npc_name: String, lines: Vec<String>) -> Self {
        Self { npc_name, lines, current_line: 0 }
    }

    pub fn current_text(&self) -> Option<&str> {
        self.lines.get(self.current_line).map(|s| s.as_str())
    }

    /// Advance to next line. Returns true if dialogue is done.
    pub fn advance(&mut self) -> bool {
        self.current_line += 1;
        self.current_line >= self.lines.len()
    }

    pub fn is_done(&self) -> bool {
        self.current_line >= self.lines.len()
    }
}
