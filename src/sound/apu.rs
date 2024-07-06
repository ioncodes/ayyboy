use std::sync::mpsc::{self, Receiver, Sender};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, OutputCallbackInfo, SampleRate, StreamConfig};

use crate::memory::mmu::Mmu;
use crate::memory::registers::AudioMasterControl;

use super::channels::{NoiseChannel, SquareChannel, WaveChannel};
use super::{MASTER_CONTROL_REGISTER, MASTER_VOLUME_REGISTER, SAMPLE_RATE};

pub struct Apu {
    pulse1: SquareChannel,
    pulse2: SquareChannel,
    wave: WaveChannel,
    noise: NoiseChannel,
    sender: Sender<f32>,
}

impl Apu {
    pub fn new(sender: Sender<f32>) -> Apu {
        Apu {
            pulse1: SquareChannel::new(),
            pulse2: SquareChannel::new(),
            wave: WaveChannel::new(),
            noise: NoiseChannel::new(),
            sender,
        }
    }

    pub fn tick(&mut self, mmu: &mut Mmu) {
        self.pulse1.tick();

        self.pulse2.update_from_registers(mmu);
        self.pulse2.tick();

        self.wave.tick();

        self.noise.tick();

        let master_volume = mmu.read_unchecked(MASTER_VOLUME_REGISTER) as f32 / 7.0;
        let master_volume = master_volume * 0.05; // TODO: lower volume artificially
        let mut amplitude = 0.0;

        let master_control = mmu.read_as_unchecked::<AudioMasterControl>(MASTER_CONTROL_REGISTER);
        if master_control.contains(AudioMasterControl::ENABLED) {
            amplitude += self.pulse1.output();
            amplitude += self.pulse2.output();
            amplitude += self.wave.output();
            amplitude += self.noise.output();
        }

        let mixed_sample = amplitude * master_volume / 4.0;
        self.sender.send(mixed_sample).expect("Failed to send sample");
    }

    pub fn setup_audio_thread() -> Sender<f32> {
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
