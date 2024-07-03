use crate::memory::mmu::Mmu;
use crate::memory::registers::InterruptFlags;
use crate::memory::{DIV_REGISTER, INTERRUPT_FLAGS_REGISTER, TAC_REGISTER, TIMA_REGISTER, TMA_REGISTER};

pub struct Timer {
    cycles: usize,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { cycles: 0 }
    }

    pub fn tick(&mut self, mmu: &mut Mmu, elapsed_cycles: usize) {
        self.cycles += elapsed_cycles;

        if self.cycles % 256 == 0 {
            let div = mmu.read_unchecked(DIV_REGISTER).wrapping_add(1);
            mmu.write_unchecked(DIV_REGISTER, div);
        }

        if self.read_tac(mmu) & 0b100 == 0 {
            return;
        }

        let tima = self.read_tima(mmu);
        let tma = self.read_tma(mmu);

        let cycles: usize = match self.read_tac(mmu) & 0b11 {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => unreachable!(),
        };

        if self.cycles % cycles == 0 {
            if tima == 0xff {
                mmu.write_unchecked(TIMA_REGISTER, tma);
                mmu.write_unchecked(
                    INTERRUPT_FLAGS_REGISTER,
                    (mmu.read_as_unchecked::<InterruptFlags>(INTERRUPT_FLAGS_REGISTER) | InterruptFlags::TIMER).bits(),
                );
            } else {
                mmu.write_unchecked(TIMA_REGISTER, tima.wrapping_add(1));
            }
        }
    }

    pub fn reset_divider(&mut self, mmu: &mut Mmu) {
        mmu.write_unchecked(DIV_REGISTER, 0);
    }

    #[inline]
    fn read_tima(&self, mmu: &Mmu) -> u8 {
        mmu.read_unchecked(TIMA_REGISTER)
    }

    #[inline]
    fn read_tma(&self, mmu: &Mmu) -> u8 {
        mmu.read_unchecked(TMA_REGISTER)
    }

    #[inline]
    fn read_tac(&self, mmu: &Mmu) -> u8 {
        mmu.read_unchecked(TAC_REGISTER)
    }
}
