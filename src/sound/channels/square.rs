use log::error;

use super::Channel;
use crate::memory::addressable::Addressable;
use crate::sound::{NR10, NR11, NR12, NR13, NR14, NR21, NR22, NR23, NR24};

const WAVE_DUTY: [[f32; 8]; 4] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // 12.5%
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // 25%
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0], // 50%
    [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0], // 75%
];

#[derive(Default)]
pub struct SquareChannel1 {
    // Tells whether the channel itself it enabled.
    // This can be only affected by a trigger event
    pub channel_enabled: bool,

    // Tells whether the channel's DAC is enabled or not
    dac_enabled: bool,

    // This is equal to `(2048 - frequency) * 4`
    // This timer is decremented every T-cycle.
    // When this timer reaches 0, wave generation is stepped, and
    // it is reloaded
    frequency_timer: u16,

    // The position we are currently in the wave
    pub wave_position: usize,

    // Sweep Time, after this time a new frequency is calculated
    sweep_period: u8,

    // Is the sweep incrementing or decrementing in nature
    sweep_is_decrementing: bool,

    // The amount by which the frequency is changed
    sweep_amount: u8,

    // The amount of sweep steps the channels has received through the
    // FS
    sweep_period_timer: u8,

    // If the sweep function is enabled or not?
    sweep_enabled: bool,

    // Stores the previous calculated frequency, depending upon some
    // conditions
    shadow_frequency: u16,

    // The wave pattern duty currently in use
    duty_pattern: u8,

    // The sound length counter. If this is >0 and bit 6 in NR24 is set
    // then it is decremented with clocks from FS. If this then hits 0
    // the sound channel is then disabled
    length_counter: u8,

    // The channel frequency value. This is controlled by NR23 and NR24
    frequency: u16,

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

impl SquareChannel1 {
    pub fn step_volume(&mut self) {
        if self.period != 0 {
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current_volume < 0xF && self.is_incrementing)
                    || (self.current_volume > 0 && !self.is_incrementing)
                {
                    if self.is_incrementing {
                        self.current_volume += 1;
                    } else {
                        self.current_volume -= 1;
                    }
                }
            }
        }
    }

    pub fn step_sweep(&mut self) {
        if self.sweep_period_timer > 0 {
            self.sweep_period_timer -= 1;
        }

        if self.sweep_period_timer == 0 {
            self.sweep_period_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };

            if self.sweep_enabled && self.sweep_period > 0 {
                let new_frequency = self.calculate_frequency();

                if new_frequency <= 2047 && self.sweep_amount > 0 {
                    self.frequency = new_frequency;
                    self.shadow_frequency = new_frequency;

                    self.calculate_frequency();
                }
            }
        }
    }

    // Calculate the new frequency, and perform the overflow check
    fn calculate_frequency(&mut self) -> u16 {
        let mut new_frequency = self.shadow_frequency >> self.sweep_amount;

        new_frequency = if self.sweep_is_decrementing {
            self.shadow_frequency - new_frequency
        } else {
            self.shadow_frequency + new_frequency
        };

        if new_frequency > 2047 {
            self.channel_enabled = false;
        }

        new_frequency
    }
}

impl Channel for SquareChannel1 {
    fn tick(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 4` and wave position is advanced by one
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 4;

            // Wave position is wrapped, so when the position is >8 it's
            // wrapped back to 0
            self.wave_position = (self.wave_position + 1) % 8;
        }

        self.frequency_timer -= 1;
    }

    // Get the current amplitude of the channel.
    // The only possible values of this are 0 or 1
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            let input = WAVE_DUTY[self.duty_pattern as usize][self.wave_position] as f32 * self.current_volume as f32;

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
                self.channel_enabled = false;
            }
        }
    }
}

#[derive(Default)]
pub struct SquareChannel2 {
    // Whether the channel itself is enabled.
    // This can be only affected by a trigger event
    pub channel_enabled: bool,

    // Whether the channel DAC is enabled or not
    dac_enabled: bool,

    // This is equal to `(2048 - frequency) * 4`
    // This timer is decremented every T-cycle.
    // When this timer reaches 0, wave generation is stepped, and
    // it is reloaded
    frequency_timer: u16,

    // The position we are currently in the wave pattern duty
    pub wave_position: usize,

    // The wave pattern duty currently in use
    duty_pattern: u8,

    // The sound length counter. If this is >0 and bit 6 in NR24 is set
    // then it is decremented with clocks from FS. If this then hits 0
    // the sound channel is then disabled
    length_counter: u8,

    // The channel frequency value. This is controlled by NR23 and NR24
    frequency: u16,

    // Whether the length timer is enabled or not
    length_enabled: bool,

    // The initial volume of the envelope function
    initial_volume: u8,

    // Whether the envelope is incrementing or decrementing in nature
    is_incrementing: bool,

    // The amount of volume steps through the FS for volume to
    // change.
    period: u8,

    // The amount of volume steps the channels has received through the
    // FS.
    period_timer: u8,

    // The current volume of the channel
    current_volume: u8,
}

impl SquareChannel2 {
    // Steps the envelope function.
    pub fn step_volume(&mut self) {
        if self.period != 0 {
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current_volume < 0xF && self.is_incrementing)
                    || (self.current_volume > 0 && !self.is_incrementing)
                {
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

impl Channel for SquareChannel2 {
    fn tick(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 4` and wave position is advanced by one
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 4;

            // Wave position is wrapped, so when the position is >8 it's
            // wrapped back to 0
            self.wave_position = (self.wave_position + 1) & 7;
        }

        self.frequency_timer -= 1;
    }

    // Get the current amplitude of the channel.
    // The only possible values of this are 0 or 1.
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            let input = WAVE_DUTY[self.duty_pattern as usize][self.wave_position] as f32 * self.current_volume as f32;

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
                self.channel_enabled = false;
            }
        }
    }
}

impl Addressable for SquareChannel1 {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        match addr {
            NR10 => {
                (self.sweep_period << 4)
                    | (if self.sweep_is_decrementing { 0x08 } else { 0x00 })
                    | self.sweep_amount
                    | 0x80
            }
            NR11 => (self.duty_pattern << 6) | 0b0011_1111,
            NR12 => (self.initial_volume << 4) | (if self.is_incrementing { 0x08 } else { 0x00 }) | self.period,
            NR14 => ((self.length_enabled as u8) << 6) | 0b1011_1111,
            _ => {
                error!("Tried to read from unmapped APU register: {:04x}", addr);
                0
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR10 => {
                // Update the sweep function parameters
                self.sweep_is_decrementing = (value & 0x08) != 0;
                self.sweep_period = value >> 4;
                self.sweep_amount = value & 0x07;
            }
            NR11 => {
                self.duty_pattern = (value >> 6) & 0b11;

                // The length counter is calculated by the following formula,
                // `Length Counter = (64 - Length Data)`
                self.length_counter = 64 - (value & 0b0011_1111);
            }
            NR12 => {
                // Update the envelope function parameters
                self.is_incrementing = (value & 0x08) != 0;
                self.initial_volume = value >> 4;
                self.period = value & 0x07;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled
                self.dac_enabled = ((value >> 3) & 0b11111) != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }
            NR13 => {
                // Update frequency with the lower eight bits
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            NR14 => {
                // Update frequency with the upper three bits
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64
                if self.length_counter == 0 {
                    self.length_counter = 64;
                }

                // Restart the channel iff DAC is enabled and trigger is set
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;

                    // Trigger the envelope function
                    self.period_timer = self.period;
                    self.current_volume = self.initial_volume;

                    // Trigger the sweep function
                    self.shadow_frequency = self.frequency;

                    // Sweep period of 0 is treated as 8 for some reason
                    self.sweep_period_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };

                    self.sweep_enabled = self.sweep_period > 0 || self.sweep_amount > 0;

                    if self.sweep_amount > 0 {
                        self.calculate_frequency();
                    }
                }
            }
            _ => error!("Tried to write to unmapped APU register: {:04x}", addr),
        }
    }
}

impl Addressable for SquareChannel2 {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        match addr {
            NR21 => (self.duty_pattern << 6) | 0b0011_1111,
            NR22 => (self.initial_volume << 4) | (if self.is_incrementing { 0x08 } else { 0x00 }) | self.period,
            NR24 => ((self.length_enabled as u8) << 6) | 0b1011_1111,
            _ => {
                error!("Tried to read from unmapped APU register: {:04x}", addr);
                0
            }
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR21 => {
                self.duty_pattern = (value >> 6) & 0b11;

                // The length counter is calculated by the following formula,
                // `Length Counter = (64 - Length Data)`
                self.length_counter = 64 - (value & 0b0011_1111);
            }
            NR22 => {
                // Update the envelope function parameters
                self.is_incrementing = (value & 0x08) != 0;
                self.initial_volume = value >> 4;
                self.period = value & 0x07;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled
                self.dac_enabled = (value & 0b1111_1000) != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }
            NR23 => {
                // Update frequency with the lower eight bits
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            NR24 => {
                // Update frequency with the upper three bits
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64
                if self.length_counter == 0 {
                    self.length_counter = 64;
                }

                // Restart the channel iff DAC is enabled and trigger is set
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;

                    // Envelope is triggered
                    self.period_timer = self.period;
                    self.current_volume = self.initial_volume;
                }
            }
            _ => error!("Tried to write to unmapped APU register: {:04x}", addr),
        }
    }
}
