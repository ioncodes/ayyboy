#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    OamScan,
    Drawing,
    HBlank,
    VBlank,
}

impl State {
    pub fn as_u8(&self) -> u8 {
        match self {
            State::OamScan => 0b10,
            State::Drawing => 0b11,
            State::HBlank => 0b00,
            State::VBlank => 0b01,
        }
    }
}
