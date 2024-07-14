use log::error;
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use super::channels::noise::NoiseChannel;
use super::channels::square::{SquareChannel1, SquareChannel2};
use super::channels::wave::WaveChannel;
use super::channels::Channel;
use super::stereo::StereoSide;
use super::{
    BUFFER_SIZE, CPU_CLOCK, NR10, NR14, NR21, NR24, NR30, NR34, NR41, NR44, NR50, NR51, NR52, SAMPLE_RATE,
    WAVE_PATTERN_RAM_END, WAVE_PATTERN_RAM_START,
};
use crate::memory::addressable::Addressable;

// TODO: Mostly taken from https://github.com/NightShade256/Argentum/

pub struct Apu {
    // The volume value for the left channel
    left_volume: u8,

    // The volume value for the right channel
    right_volume: u8,

    // $FF25 - Controls which stereo channels, sound is outputted to
    nr51: u8,

    // APU enabled - Controls whether the APU is ticking
    apu_enabled: bool,

    // Implementation of the square wave channel one with envelope and sweep function
    square1: SquareChannel1,

    // Implementation of the square wave channel two with envelope function
    square2: SquareChannel2,

    // Implementation of the custom wave channel
    wave: WaveChannel,

    // Implementation of the noise wave channel
    noise: NoiseChannel,

    // Used to clock FS and sample generation
    sample_clock: usize,

    // Current CPU clock rate
    cpu_clock: usize,

    // The audio buffer which contains 32-bit float samples
    pub buffer: [f32; BUFFER_SIZE],

    // The position we are currently in the audio buffer
    pub buffer_position: usize,

    // The position the FS is currently in
    frame_sequencer_position: u8,

    // Stub
    left_vin: bool,

    // Stub
    right_vin: bool,

    // Output stream sink
    audio_sink: Sink,

    // Output stream, we need to keep this alive
    _stream: OutputStream,
}

impl Apu {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let audio_sink = Sink::try_new(&stream_handle).unwrap();

        Self {
            left_volume: 0,
            right_volume: 0,
            nr51: 0,
            apu_enabled: false,
            square1: SquareChannel1::default(),
            square2: SquareChannel2::default(),
            wave: WaveChannel::default(),
            noise: NoiseChannel::default(),
            sample_clock: 0,
            cpu_clock: CPU_CLOCK,
            buffer: [0.0; BUFFER_SIZE],
            buffer_position: 0,
            frame_sequencer_position: 0,
            left_vin: false,
            right_vin: false,
            audio_sink,
            _stream: stream,
        }
    }

    pub fn push_samples(&self, buffer: &[f32]) {
        while self.audio_sink.len() > 2 {
            // Wait for the sink to have played enough samples
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        self.audio_sink
            .append(SamplesBuffer::new(2, SAMPLE_RATE as u32, buffer));
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            // This clock is incremented every T-cycle.
            // This is used to clock the frame sequencer and
            // to generate sample
            self.sample_clock = self.sample_clock.wrapping_add(1);

            // Tick all the connected channels
            self.square1.tick();
            self.square2.tick();
            self.wave.tick();
            self.noise.tick();

            // Tick the frame sequencer. It generates clocks for the length,
            // envelope and sweep functions
            if self.sample_clock % 8192 == 0 {
                self.clock_components();
                self.frame_sequencer_position = (self.frame_sequencer_position + 1) % 8;
                self.sample_clock = 0;
            }

            // Each (CPU CLOCK / SAMPLE RATE) cycles one sample is generated
            // and pushed to the buffer
            if self.sample_clock % (self.cpu_clock / SAMPLE_RATE) == 0 {
                let left_amplitude = self.get_amplitude_for_channel(0, StereoSide::Left)
                    + self.get_amplitude_for_channel(1, StereoSide::Left)
                    + self.get_amplitude_for_channel(2, StereoSide::Left)
                    + self.get_amplitude_for_channel(3, StereoSide::Left);
                let right_amplitude = self.get_amplitude_for_channel(0, StereoSide::Right)
                    + self.get_amplitude_for_channel(1, StereoSide::Right)
                    + self.get_amplitude_for_channel(2, StereoSide::Right)
                    + self.get_amplitude_for_channel(3, StereoSide::Right);

                self.buffer[self.buffer_position + 0] = (self.left_volume as f32 / 7.0) * left_amplitude / 4.0;
                self.buffer[self.buffer_position + 1] = (self.right_volume as f32 / 7.0) * right_amplitude / 4.0;

                self.buffer_position += 2;
            }

            // Checks if the buffer is full and pushes samples to audio sink
            if self.buffer_position >= BUFFER_SIZE {
                self.push_samples(self.buffer.as_ref());
                self.buffer_position = 0;
            }
        }
    }

    pub fn update_cpu_clock(&mut self, cpu_clock: usize) {
        self.cpu_clock = cpu_clock;
    }

    pub fn reset_cpu_clock(&mut self) {
        self.cpu_clock = CPU_CLOCK;
    }

    fn clock_components(&mut self) {
        // https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
        match self.frame_sequencer_position {
            0 => {
                self.square1.step_length();
                self.square2.step_length();
                self.wave.step_length();
                self.noise.step_length();
            }
            2 => {
                self.square1.step_length();
                self.square2.step_length();
                self.wave.step_length();
                self.noise.step_length();
                self.square1.step_sweep();
            }
            4 => {
                self.square1.step_length();
                self.square2.step_length();
                self.wave.step_length();
                self.noise.step_length();
            }
            6 => {
                self.square1.step_length();
                self.square2.step_length();
                self.wave.step_length();
                self.noise.step_length();
                self.square1.step_sweep();
            }
            7 => {
                self.square1.step_volume();
                self.square2.step_volume();
                self.noise.step_volume();
            }
            _ => {}
        }
    }

    fn get_amplitude_for_channel(&self, channel: u8, side: StereoSide) -> f32 {
        // Tries to get the amplitude for the given channel and side
        // If the bit is not set in NR51, the channel does not go to the
        // given side

        let channel_offset = match side {
            StereoSide::Right => 0,
            StereoSide::Left => 4,
        };

        let enabled = (self.nr51 & (1 << (channel + channel_offset))) != 0;

        match channel {
            0 if enabled => self.square1.get_amplitude(),
            1 if enabled => self.square2.get_amplitude(),
            2 if enabled => self.wave.get_amplitude(),
            3 if enabled => self.noise.get_amplitude(),
            _ => 0.0,
        }
    }
}

impl Addressable for Apu {
    #[inline]
    fn read(&self, addr: u16) -> u8 {
        match addr {
            NR50 => {
                (if self.left_vin { 0b1000_0000 } else { 0 })
                    | (self.left_volume << 4)
                    | (if self.right_vin { 0b0000_1000 } else { 0 })
                    | self.right_volume
            }
            NR51 => self.nr51,
            NR52 => {
                let mut nr52 = ((self.apu_enabled as u8) << 7) | 0x70;

                nr52 |= self.square1.channel_enabled as u8;
                nr52 |= (self.square2.channel_enabled as u8) << 1;
                nr52 |= (self.wave.channel_enabled as u8) << 2;
                nr52 |= (self.noise.enabled as u8) << 3;

                nr52
            }

            NR10..=NR14 => self.square1.read(addr),
            NR21..=NR24 => self.square2.read(addr),
            NR30..=NR34 | WAVE_PATTERN_RAM_START..=WAVE_PATTERN_RAM_END => self.wave.read(addr),
            NR41..=NR44 => self.noise.read(addr),

            _ => unreachable!(),
        }
    }

    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR50 => {
                self.left_volume = (value >> 4) & 0x07;
                self.right_volume = value & 0x07;

                self.left_vin = (value & 0b1000_0000) != 0;
                self.right_vin = (value & 0b0000_1000) != 0;
            }
            NR51 => self.nr51 = value,
            NR52 => {
                let enabled = (value >> 7) != 0;

                if !enabled && self.apu_enabled {
                    for addr in NR10..=NR51 {
                        self.write(addr, 0x00);
                    }

                    self.apu_enabled = false;
                } else if enabled && !self.apu_enabled {
                    self.apu_enabled = true;

                    self.frame_sequencer_position = 0;

                    self.square1.wave_position = 0;
                    self.square2.wave_position = 0;
                    self.wave.wave_position = 0;
                }
            }
            NR10..=NR14 => self.square1.write(addr, value),
            NR21..=NR24 => self.square2.write(addr, value),
            NR30..=NR34 | WAVE_PATTERN_RAM_START..=WAVE_PATTERN_RAM_END => self.wave.write(addr, value),
            NR41..=NR44 => self.noise.write(addr, value),
            _ => error!("Tried to write to unmapped APU register: {:04x}", addr),
        }
    }
}
