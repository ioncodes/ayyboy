use eframe::egui::Key;
use log::warn;

#[derive(Clone)]
pub struct Joypad {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            start: false,
            select: false,
        }
    }

    pub fn update_button(&mut self, key: Key, pressed: bool) {
        match key {
            Key::ArrowUp => self.up = pressed,
            Key::ArrowDown => self.down = pressed,
            Key::ArrowLeft => self.left = pressed,
            Key::ArrowRight => self.right = pressed,
            Key::A => self.a = pressed,
            Key::S => self.b = pressed,
            Key::Enter => self.start = pressed,
            Key::Backspace => self.select = pressed,
            _ => unreachable!(),
        }
    }

    pub fn as_u8(&self, joypad_state: u8) -> u8 {
        let button_select = joypad_state & 0b0010_0000 == 0;
        let direction_select = joypad_state & 0b0001_0000 == 0;
        if button_select && direction_select {
            warn!("Joypad has buttons and d-pad mode selected");
        }

        let mut state = joypad_state & 0b1111_0000;

        if button_select {
            if self.start {
                state |= 0b0000_1000;
            }
            if self.select {
                state |= 0b0000_0100;
            }
            if self.b {
                state |= 0b0000_0010;
            }
            if self.a {
                state |= 0b0000_0001;
            }
        } else if direction_select {
            if self.down {
                state |= 0b0000_1000;
            }
            if self.up {
                state |= 0b0000_0100;
            }
            if self.left {
                state |= 0b0000_0010;
            }
            if self.right {
                state |= 0b0000_0001;
            }
        }

        !state
    }
}
