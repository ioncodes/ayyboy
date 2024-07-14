#[derive(Copy, Clone, PartialEq)]
pub enum State {
    HBlank,  // H-Blank
    VBlank,  // V-Blank
    OamScan, // OAM Scan
    Drawing, // Drawing
}

impl State {
    pub fn as_u8(self) -> u8 {
        match self {
            State::HBlank => 0,
            State::VBlank => 1,
            State::OamScan => 2,
            State::Drawing => 3,
        }
    }
}
