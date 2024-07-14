use log::error;

use crate::memory::addressable::Addressable;
use crate::sound::{NR30, NR31, NR32, NR33, NR34, WAVE_PATTERN_RAM_END, WAVE_PATTERN_RAM_START};

use super::Channel;

#[derive(Default, Clone)]
pub struct WaveChannel {
    // Whether the channel itself it enabled.
    // This can be only affected by a trigger event
    pub channel_enabled: bool,

    // Whether the channel DAC is enabled or not
    dac_enabled: bool,

    // This is equal to `(2048 - frequency) * 2`
    // This timer is decremented every T-cycle.
    // When this timer reaches 0, wave generation is stepped, and
    // it is reloaded
    frequency_timer: u16,

    // The current sample being played in the wave ram
    pub wave_position: usize,

    // The sound length counter. If this is >0 and bit 6 in NR24 is set
    // then it is decremented with clocks from FS. If this then hits 0
    // the sound channel is then disabled
    length_counter: u16,

    // Output level configuration register.
    // Sets the volume shift for the wave data
    output_level: u8,

    // The volume shift computed from the output level
    // register
    volume_shift: u8,

    // The channel frequency value. This is controlled by NR23 and NR24
    frequency: u16,

    // Whether the length timer is enabled or not
    length_enabled: bool,

    // Arbitrary 32 4-bit samples
    wave_ram: [u8; 0x10],
}

impl Channel for WaveChannel {
    // Tick the channel by one T-cycle
    fn tick(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 2` and wave position is advanced by one
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 2;

            // Wave position is wrapped, so when the position is >32 it's
            // wrapped back to 0
            self.wave_position = (self.wave_position + 1) & 31;
        }

        self.frequency_timer -= 1;
    }

    // Get the current amplitude of the channel
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled {
            let sample =
                ((self.wave_ram[self.wave_position / 2]) >> (if (self.wave_position & 1) != 0 { 4 } else { 0 })) & 0x0F;

            (((sample >> self.volume_shift) as f32) / 7.5) - 1.0
        } else {
            0.0
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;

            // The channel is disabled if the length counter is reset.
            if self.length_counter == 0 {
                self.channel_enabled = false;
            }
        }
    }
}

impl Addressable for WaveChannel {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        match addr {
            NR30 => ((self.dac_enabled as u8) << 7) | 0x7F,
            NR32 => (self.output_level << 5) | 0x9F,
            NR34 => ((self.length_enabled as u8) << 6) | 0b1011_1111,
            WAVE_PATTERN_RAM_START..=WAVE_PATTERN_RAM_END => self.wave_ram[(addr - WAVE_PATTERN_RAM_START) as usize],
            _ => {
                error!("Unimplemented read from APU register: {:04x}", addr);
                0
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR30 => {
                self.dac_enabled = (value >> 7) & 0b1 != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }
            NR31 => {
                self.length_counter = 256 - value as u16;
            }
            NR32 => {
                self.output_level = (value >> 5) & 0b11;

                self.volume_shift = match self.output_level {
                    0b00 => 4,
                    0b01 => 0,
                    0b10 => 1,
                    0b11 => 2,

                    _ => unreachable!(),
                };
            }
            NR33 => {
                // Update frequency with the lower eight bits
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            NR34 => {
                // Update frequency with the upper three bits
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 6
                if self.length_counter == 0 {
                    self.length_counter = 256;
                }

                // Restart the channel iff DAC is enabled and trigger is set
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;
                }
            }
            WAVE_PATTERN_RAM_START..=WAVE_PATTERN_RAM_END => {
                self.wave_ram[(addr - WAVE_PATTERN_RAM_START) as usize] = value
            }
            _ => error!("Unimplemented write to APU register: {:04x}", addr),
        }
    }
}
