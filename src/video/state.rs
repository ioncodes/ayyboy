#[derive(Clone, Copy, PartialEq)]
pub enum State {
    OamScan,
    Drawing,
    HBlank,
    VBlank,
}
