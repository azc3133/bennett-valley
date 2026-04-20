use macroquad::prelude::*;
use crate::game::player::Direction;

#[derive(Debug, Clone)]
pub enum InputEvent {
    Move(Direction),
    Hoe,
    Water,
    Plant,
    Harvest,
    Forage,
    Fish,
    Mine,
    /// T: Gift the best matching item to the facing NPC.
    Gift,
    /// I: Toggle the relationships overview screen.
    ToggleRelationships,
    Interact,
    Sleep,
    CloseMenu,
    ShopUp,
    ShopDown,
    SelectSeedParsnip,
    SelectSeedPotato,
    SelectSeedCauliflower,
    Propose,
    /// R: Begin replying to the open letter.
    Reply,
    /// Delete: Show reset-game confirmation overlay.
    ResetGame,
    /// J: Mount/dismount horse.
    ToggleHorse,
    /// Space: Horse leap.
    HorseLeap,
    /// O: Open outfit shop / wardrobe.
    OpenOutfits,
    /// N: P2 action (alternative to Enter/Period for arrow key issues).
    P2Action,
    /// X: Scythe long grass.
    Scythe,
    /// F1: Toggle background music.
    ToggleMusic,
    /// F2: Toggle co-op mode.
    ToggleCoop,
    /// F3: Host multiplayer room.
    MpHost,
    /// F4: Join multiplayer room.
    MpJoin,
}

/// Action-only events (is_key_pressed — fires once per key press).
/// Movement is handled separately in main.rs using is_key_down.
pub fn collect_action_events() -> Vec<InputEvent> {
    let mut events = Vec::new();

    if is_key_pressed(KeyCode::H) {
        events.push(InputEvent::Hoe);
    }
    if is_key_pressed(KeyCode::F) {
        events.push(InputEvent::Water);
    }
    if is_key_pressed(KeyCode::P) {
        events.push(InputEvent::Plant);
    }
    if is_key_pressed(KeyCode::R) {
        events.push(InputEvent::Harvest);
    }
    if is_key_pressed(KeyCode::G) {
        events.push(InputEvent::Forage);
    }
    if is_key_pressed(KeyCode::T) {
        events.push(InputEvent::Gift);
    }
    if is_key_pressed(KeyCode::I) {
        events.push(InputEvent::ToggleRelationships);
    }
    if is_key_pressed(KeyCode::C) {
        events.push(InputEvent::Fish);
    }
    if is_key_pressed(KeyCode::M) {
        events.push(InputEvent::Mine);
    }
    if is_key_pressed(KeyCode::E) {
        events.push(InputEvent::Interact);
    }
    if is_key_pressed(KeyCode::Z) {
        events.push(InputEvent::Sleep);
    }
    if is_key_pressed(KeyCode::Escape) {
        events.push(InputEvent::CloseMenu);
    }
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        events.push(InputEvent::ShopUp);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        events.push(InputEvent::ShopDown);
    }
    if is_key_pressed(KeyCode::B) {
        events.push(InputEvent::Propose);
    }
    if is_key_pressed(KeyCode::R) {
        events.push(InputEvent::Reply);
    }
    if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
        events.push(InputEvent::ResetGame);
    }
    if is_key_pressed(KeyCode::J) {
        events.push(InputEvent::ToggleHorse);
    }
    if is_key_pressed(KeyCode::Space) {
        events.push(InputEvent::HorseLeap);
    }
    if is_key_pressed(KeyCode::O) {
        events.push(InputEvent::OpenOutfits);
    }
    if is_key_pressed(KeyCode::X) {
        events.push(InputEvent::Scythe);
    }
    if is_key_pressed(KeyCode::N) {
        events.push(InputEvent::P2Action);
    }
    if is_key_pressed(KeyCode::F1) || is_key_pressed(KeyCode::Minus) {
        events.push(InputEvent::ToggleMusic);
    }
    if is_key_pressed(KeyCode::F2) || is_key_pressed(KeyCode::GraveAccent) {
        events.push(InputEvent::ToggleCoop);
    }
    if is_key_pressed(KeyCode::F3) {
        events.push(InputEvent::MpHost);
    }
    if is_key_pressed(KeyCode::F4) {
        events.push(InputEvent::MpJoin);
    }
    if is_key_pressed(KeyCode::Key1) {
        events.push(InputEvent::SelectSeedParsnip);
    }
    if is_key_pressed(KeyCode::Key2) {
        events.push(InputEvent::SelectSeedPotato);
    }
    if is_key_pressed(KeyCode::Key3) {
        events.push(InputEvent::SelectSeedCauliflower);
    }
    events
}
