use crate::memory::mmu::Mmu;

use super::{NR21, NR22, NR23, NR24, SAMPLE_RATE};

pub struct SquareChannel {
    frequency: u16,
    duty: u8,
    volume: f32,
    phase: f32,
    length_counter: u16,
    enabled: bool,
    frequency_timer: u16,
    wave_duty_position: u8,
}

pub struct WaveChannel {}
pub struct NoiseChannel {}

impl SquareChannel {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            duty: 2,
            volume: 0.5,
            phase: 0.0,
            length_counter: 0,
            enabled: false,
            frequency_timer: 0,
            wave_duty_position: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.enabled {
            if self.frequency_timer == 0 {
                self.frequency_timer = (2048 - self.frequency) * 4;
                self.wave_duty_position = (self.wave_duty_position + 1) % 8;
            } else {
                self.frequency_timer -= 1;
            }

            // Decrement length counter
            if self.length_counter > 0 {
                self.length_counter = self.length_counter.saturating_sub(1);
                if self.length_counter == 0 {
                    self.enabled = false; // Disable channel when length counter reaches zero
                }
            }
        }
    }

    pub fn output(&self) -> f32 {
        if self.enabled {
            let duty_cycle = match self.duty {
                0 => [0, 0, 0, 0, 0, 0, 0, 1],
                1 => [1, 0, 0, 0, 0, 0, 0, 1],
                2 => [1, 0, 0, 0, 0, 1, 1, 1],
                3 => [0, 1, 1, 1, 1, 1, 1, 0],
                _ => [0, 0, 0, 0, 0, 0, 0, 0], // Default case should never happen
            };

            if duty_cycle[self.wave_duty_position as usize] == 1 {
                self.volume
            } else {
                -self.volume
            }
        } else {
            0.0
        }
    }

    pub fn update_from_registers(&mut self, mmu: &Mmu) {
        self.duty = (mmu.read_unchecked(NR21) & 0xC0) >> 6;
        self.volume = (mmu.read_unchecked(NR22) & 0xF0) as f32 / 0xF0 as f32;

        let lower_bits = mmu.read_unchecked(NR23) as u16;
        let upper_bits = (mmu.read_unchecked(NR24) as u16 & 0x07) << 8;
        self.frequency = lower_bits | upper_bits;

        // Update length counter and enable flag from NR24
        if mmu.read_unchecked(NR24) & 0x80 != 0 {
            self.enabled = true;
            self.length_counter = 64 - (mmu.read_unchecked(NR21) & 0x3F) as u16;
        }
    }
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self) {}

    pub fn output(&self) -> f32 {
        0.0
    }
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self) {}

    pub fn output(&self) -> f32 {
        0.0
    }
}
