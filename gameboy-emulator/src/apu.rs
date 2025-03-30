use crate::memory::Memory;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const BIT_7_MASK: u8 = 1 << 7;

pub struct APU {
    pulse_period_divider: u16,
    wave_period_divider: u16,
    ch1_freq: Arc<Mutex<Option<f32>>>,
    handle: thread::JoinHandle<()>,
}

impl APU {
    pub fn new() -> Self {

        let freq: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        let freq_clone = freq.clone();
        let handle = thread::spawn(move || {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .expect("no audio output device available");
            let mut supported_configs_range = device
                .supported_output_configs()
                .expect("error while querying configs");
            let supported_config = supported_configs_range
                .next()
                .expect("no supported config?!")
                .with_max_sample_rate();
            let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
            let sample_rate = supported_config.sample_rate().0 as f32;
            let config = supported_config.into();
            let mut sample_clock = 0.0;

            let stream = {
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], c: &cpal::OutputCallbackInfo| {
                            let mut f = freq.lock().unwrap();
                            match &mut *f {
                                None => {
                                    for sample in data.iter_mut() {
                                        *sample = Sample::EQUILIBRIUM;
                                    }
                                }
                                Some(f) => {
                                    for sample in data.iter_mut() {
                                        sample_clock = (sample_clock + 1.0) % sample_rate;
                                        let amp =
                                            sample_clock * f.clone() * 2.0 * std::f32::consts::PI
                                                / sample_rate;
                                        *sample = if amp.sin() > 0.0 { 0.5 } else { -0.5 };
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                    .unwrap()
            };
            stream.play().unwrap();
            sleep(Duration::from_secs(5000))
        });
        Self {
            pulse_period_divider: 0,
            wave_period_divider: 0,
            ch1_freq: freq_clone,
            handle,
        }
    }

    pub fn clock_pulse(&mut self, mem: &mut Memory) {
        self.pulse_period_divider += 1;

        let ch1_period_lo = mem[0xFF13];
        let ch1_period_hi = mem[0xFF14];
        let ch1_period = (((ch1_period_hi & 0b111) as u16) << 8) | (ch1_period_lo as u16);
        if ch1_period_hi & BIT_7_MASK != 0 {
            // TODO do trigger things here
            mem[0xFF14] &= !BIT_7_MASK;
        }

        if self.pulse_period_divider >= 0x7FF {
            self.pulse_period_divider = ch1_period;
            self.ch1_freq
                .lock()
                .unwrap()
                .replace(131072.0 / (2048 - ch1_period) as f32);
        }
    }

    pub fn clock_wave(&mut self, mem: &mut Memory) {
        self.wave_period_divider += 1;
    }

    pub fn update(&mut self, mem: &mut Memory) {
        let master_ctrl = mem[0xFF26];
        let panning = mem[0xFF25];
        let volume_vin = mem[0xFF24];

        let on_off = (master_ctrl & BIT_7_MASK) != 0;
    }
}
