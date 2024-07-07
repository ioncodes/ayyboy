use crate::memory::mmu::Mmu;

use super::{NR11, NR12, NR13, NR14, NR21, NR22, NR23, NR24, SAMPLE_RATE};

const DUTY_CYCLES: [[f32; 8]; 4] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // 12.5% duty cycle
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0], // 25% duty cycle
    [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0], // 50% duty cycle
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0], // 75% duty cycle
];

#[derive(Clone)]
pub struct SquareChannel1 {
    pub wave_duty: usize,
    pub wave_duty_position: usize,
    pub length_counter: u8,
    pub length_counter_enabled: bool,
    pub volume: f32,
    pub envelope_increase: bool,
    pub sweep_pace: u8,
    pub frequency: u16,
    pub frequency_timer: u16,
    pub enabled: bool,
    // Internal state
}

impl SquareChannel1 {
    pub fn new() -> SquareChannel1 {
        SquareChannel1 {
            wave_duty: 0,
            wave_duty_position: 0,
            length_counter: 0,
            length_counter_enabled: false,
            volume: 0.0,
            envelope_increase: false,
            sweep_pace: 0,
            frequency: 0,
            enabled: false,
            frequency_timer: 0,
        }
    }

    pub fn tick(&mut self) {
        self.frequency_timer -= 1;
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 4;
            self.wave_duty_position = (self.wave_duty_position + 1) % 8;
        }
    }

    pub fn tick_length_counter(&mut self) {
        if self.length_counter_enabled && self.length_counter > 0 {
            self.length_counter -= 1;

            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn sample(&mut self) -> f32 {
        if !self.enabled || self.frequency_timer == 0 {
            return 0.0;
        }

        let duty_pattern = DUTY_CYCLES[self.wave_duty];
        let amplitude = duty_pattern[self.wave_duty_position] * self.volume;
        (amplitude / 7.5) - 1.0
    }
}
