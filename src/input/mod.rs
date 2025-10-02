//! Gestion des contrôles et entrées

use std::collections::HashSet;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

/// Gestionnaire d'entrées
#[derive(Debug)]
pub struct InputManager {
    pressed_keys: HashSet<KeyCode>,
    pub player1: PlayerInput,
    pub player2: PlayerInput,
}

/// Entrées d'un joueur
#[derive(Debug, Clone, Default)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub punch: bool,
    pub kick: bool,
    pub guard: bool,
    pub start: bool,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            player1: PlayerInput::default(),
            player2: PlayerInput::default(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.pressed_keys.insert(key);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&key);
            }
        }
        self.update_player_inputs();
    }

    fn update_player_inputs(&mut self) {
        // Player 1 (WASD + touches)
        self.player1.up = self.pressed_keys.contains(&KeyCode::KeyW);
        self.player1.down = self.pressed_keys.contains(&KeyCode::KeyS);
        self.player1.left = self.pressed_keys.contains(&KeyCode::KeyA);
        self.player1.right = self.pressed_keys.contains(&KeyCode::KeyD);
        self.player1.punch = self.pressed_keys.contains(&KeyCode::KeyJ);
        self.player1.kick = self.pressed_keys.contains(&KeyCode::KeyK);
        self.player1.guard = self.pressed_keys.contains(&KeyCode::KeyL);
        self.player1.start = self.pressed_keys.contains(&KeyCode::Enter);

        // Player 2 (flèches + numpad)
        self.player2.up = self.pressed_keys.contains(&KeyCode::ArrowUp);
        self.player2.down = self.pressed_keys.contains(&KeyCode::ArrowDown);
        self.player2.left = self.pressed_keys.contains(&KeyCode::ArrowLeft);
        self.player2.right = self.pressed_keys.contains(&KeyCode::ArrowRight);
        self.player2.punch = self.pressed_keys.contains(&KeyCode::Numpad1);
        self.player2.kick = self.pressed_keys.contains(&KeyCode::Numpad2);
        self.player2.guard = self.pressed_keys.contains(&KeyCode::Numpad3);
        self.player2.start = self.pressed_keys.contains(&KeyCode::NumpadEnter);
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
