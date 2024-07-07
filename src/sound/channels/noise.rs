use log::error;

use crate::memory::addressable::Addressable;
use crate::sound::{NR41, NR42, NR43, NR44};

use super::Channel;

#[derive(Default, Clone)]
pub struct NoiseChannel {
    // Tells whether the channel itself it enabled.
    // This can be only affected by the `length` parameter
    pub enabled: bool,

    // Whether the channel DAC is enabled or not
    dac_enabled: bool,

    // This is equal to `(2048 - frequency) * 2`
    // This timer is decremented every T-cycle.
    // When this timer reaches 0, wave generation is stepped, and
    // it is reloaded
    frequency_timer: u16,

    // The linear feedback shift register (LFSR) generates a pseudo-random bit sequence
    lfsr: u16,

    // The sound length counter. If this is >0 and bit 6 in NR24 is set
    // then it is decremented with clocks from FS. If this then hits 0
    // the sound channel is then disabled
    length_counter: u8,

    // The polynomial counter, used to control the RNG
    nr43: u8,

    // Whether the length timer is enabled or not
    length_enabled: bool,

    // The initial volume of the envelope function
    initial_volume: u8,

    // Whether the envelope is incrementing or decrementing in nature
    is_incrementing: bool,

    // The amount of volume steps through the FS for volume to
    // change
    period: u8,

    // The amount of volume steps the channels has received through the
    // FS
    period_timer: u8,

    // The current volume of the channel
    current_volume: u8,
}

impl NoiseChannel {
    pub fn step_volume(&mut self) {
        if self.period != 0 {
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current_volume < 0xF && self.is_incrementing) || (self.current_volume > 0 && !self.is_incrementing) {
                    if self.is_incrementing {
                        self.current_volume += 1;
                    } else {
                        self.current_volume -= 1;
                    }
                }
            }
        }
    }
}

impl Channel for NoiseChannel {
    fn tick(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `divisor_code << clock_shift` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            let divisor_code = (self.nr43 & 0x07) as u16;

            self.frequency_timer = (if divisor_code == 0 { 8 } else { divisor_code << 4 }) << ((self.nr43 >> 4) as u32);

            let xor_result = (self.lfsr & 0b01) ^ ((self.lfsr & 0b10) >> 1);

            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

            if ((self.nr43 >> 3) & 0b01) != 0 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result << 6;
            }
        }

        self.frequency_timer = self.frequency_timer.wrapping_sub(1);
    }

    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled && self.enabled {
            let input = (!self.lfsr & 0b01) as f32 * self.current_volume as f32;

            (input / 7.5) - 1.0
        } else {
            0.0
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;

            // The channel is disabled if the length counter is reset.
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
}

impl Addressable for NoiseChannel {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        match addr {
            NR42 => (self.initial_volume << 4) | (if self.is_incrementing { 0x08 } else { 0x00 }) | self.period,
            NR43 => self.nr43,
            NR44 => ((self.length_enabled as u8) << 6) | 0b1011_1111,
            _ => {
                error!("Unimplemented read from APU register: {:04x}", addr);
                0
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR41 => self.length_counter = 64 - (value & 0b0011_1111),
            NR42 => {
                // Update the envelope function parameters
                self.is_incrementing = (value & 0x08) != 0;
                self.initial_volume = value >> 4;
                self.period = value & 0x07;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled
                self.dac_enabled = (value & 0b1111_1000) != 0;

                if !self.dac_enabled {
                    self.enabled = false;
                }
            }
            NR43 => self.nr43 = value,
            NR44 => {
                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64.
                if self.length_counter == 0 {
                    self.length_counter = 64;
                }

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.enabled = true;
                }

                if trigger {
                    // On trigger event all bits of LFSR are turned on.
                    self.lfsr = 0x7FFF;

                    // Envelope is triggered.
                    self.period_timer = self.period;
                    self.current_volume = self.initial_volume;
                }
            }
            _ => error!("Tried to write to unmapped APU register: {:04x}", addr),
        }
    }
}
