use std::sync::mpsc::{self, Receiver, Sender};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, OutputCallbackInfo, SampleRate, StreamConfig};
use log::error;

use crate::memory::addressable::Addressable;
use crate::memory::mmu::Mmu;
use crate::memory::registers::AudioMasterControl;

use super::channels::SquareChannel1;
use super::{NR11, NR12, NR13, NR14, NR50, NR52, SAMPLE_RATE};

#[derive(Clone)]
pub struct Apu {
    pulse1: SquareChannel1,
    cycles: usize,
    sampler: Sender<f32>,
    master_volume: f32,
    enabled: bool,
}

impl Apu {
    pub fn new() -> Apu {
        let sampler = Apu::setup_audio_thread();

        Apu {
            pulse1: SquareChannel1::new(),
            cycles: 0,
            sampler,
            master_volume: 1.0,
            enabled: false,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.pulse1.tick();
        }

        if !self.enabled {
            return;
        }

        if self.cycles < 8192 {
            self.cycles += cycles;
            return;
        }

        self.cycles -= 8192;

        let mut amplitude = 0.0;
        if self.pulse1.enabled {
            amplitude += self.pulse1.sample();
        }

        let sample = amplitude * self.master_volume;
        if sample > 0.0 {
            println!("Sample: {}", sample);
        }
        self.sampler.send(sample).expect("Failed to send sample");
    }

    fn setup_audio_thread() -> Sender<f32> {
        let (sender, receiver): (Sender<f32>, Receiver<f32>) = mpsc::channel();

        std::thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_output_device().expect("No output device available");
            let config = StreamConfig {
                channels: 1,
                sample_rate: SampleRate(SAMPLE_RATE),
                buffer_size: BufferSize::Default,
            };

            let stream = device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &OutputCallbackInfo| {
                        for sample in data.iter_mut() {
                            *sample = receiver.recv().unwrap_or(0.0);
                        }
                    },
                    |err| eprintln!("Error during audio playback: {}", err),
                    None,
                )
                .unwrap();

            stream.play().unwrap();
            loop {}
        });

        sender
    }
}

impl Addressable for Apu {
    #[inline]
    fn read(&self, address: u16) -> u8 {
        match address {
            NR52 => {
                let mut master_control = AudioMasterControl::from_bits_truncate(0);
                if self.enabled {
                    master_control.insert(AudioMasterControl::ENABLED);
                }

                master_control.bits()
            }
            _ => {
                error!("Attempted to read from unmapped APU register: {:04x}", address);
                0
            }
        }
    }

    #[inline]
    fn write(&mut self, address: u16, data: u8) {
        match address {
            NR11 => {
                self.pulse1.frequency_timer = (data & 0b0011_1111) as u16;
                self.pulse1.wave_duty = ((data & 0b1100_0000) >> 6) as usize;
            }
            NR12 => {
                self.pulse1.volume = ((data & 0b1111_0000) >> 4) as f32;
                self.pulse1.envelope_increase = data & 0b0000_1000 != 0;
                self.pulse1.sweep_pace = data & 0b0000_0111;
            }
            NR13 => {
                self.pulse1.frequency = (self.pulse1.frequency & 0x700) | data as u16;
            }
            NR14 => {
                if data & 0b1000_0000 != 0 {
                    self.pulse1.enabled = true;
                }
                if data & 0b0100_0000 != 0 {
                    self.pulse1.frequency = (self.pulse1.frequency & 0xFF) | ((data as u16 & 0b0000_0111) << 8);
                    self.pulse1.frequency_timer = 64 - self.pulse1.frequency_timer;
                }
                // TODO: period?
            }
            NR50 => {
                // TODO: technically we should have 2 channels for stereo sound
                let mut left_volume = ((data & 0b0111_0000) >> 4) as f32;
                if left_volume == 0.0 {
                    left_volume = 1.0;
                }

                let mut right_volume = (data & 0b0000_0111) as f32;
                if right_volume == 0.0 {
                    right_volume = 1.0;
                }

                // Scale volume to 0.0 - 1.0
                self.master_volume = (left_volume + right_volume) / 7.0;
            }
            NR52 => {
                let master_control = AudioMasterControl::from_bits_truncate(data);
                self.enabled = master_control.contains(AudioMasterControl::ENABLED);
            }
            _ => {
                error!("Attempted to write to unmapped APU register: {:04x} with {:02x}", address, data);
            }
        }
    }
}
