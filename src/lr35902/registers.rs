use bitflags::bitflags;

bitflags! {
    #[derive(Clone)]
    pub struct Flags: u8 {
        const ZERO       = 0b1000_0000;
        const SUBTRACT   = 0b0100_0000;
        const HALF_CARRY = 0b0010_0000;
        const CARRY      = 0b0001_0000;
    }
}

#[derive(Clone)]
pub struct Registers {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            a: 0,
            f: Flags::empty(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
}
