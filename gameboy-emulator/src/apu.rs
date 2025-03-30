use crate::memory::Memory;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const BIT_7_MASK: u8 = 1 << 7;

struct PulseChannel {
    enabled: bool,
    freq: f32,
    duty: u8,
    initial_length: u8,
    length_timer: u8,
    period_divider: u16,
}

impl PulseChannel {
    fn set_pulse_divider(&mut self, mem: &Memory) {
        let ch1_period_lo = mem[0xFF13];
        let ch1_period_hi = mem[0xFF14];
        let ch1_period = (((ch1_period_hi & 0b111) as u16) << 8) | (ch1_period_lo as u16);
        self.period_divider = ch1_period;
        self.freq = 131072.0 / (2048 - ch1_period) as f32;
    }
}

pub struct APU {
    wave_period_divider: u16,
    ch1: Arc<Mutex<PulseChannel>>,
    handle: thread::JoinHandle<()>,
}

impl APU {
    pub fn new() -> Self {
        let ch1 = Arc::new(Mutex::new(PulseChannel {
            enabled: false,
            freq: 0.0,
            duty: 0,
            initial_length: 0,
            length_timer: 0,
            period_divider: 0,
        }));

        let ch1_clone = ch1.clone();
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
                            let mut c = ch1.lock().unwrap();
                            if !c.enabled {
                                for sample in data.iter_mut() {
                                    *sample = Sample::EQUILIBRIUM;
                                }
                            } else {
                                let bound = 1.0 / (2 * c.duty as i8 - 2) as f32;
                                for sample in data.iter_mut() {
                                    sample_clock = (sample_clock + 1.0) % sample_rate;
                                    let amp = sample_clock * c.freq * 2.0 * std::f32::consts::PI
                                        / sample_rate;
                                    *sample = if amp.sin() > bound { 0.5 } else { -0.5 };
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
            wave_period_divider: 0,
            ch1: ch1_clone,
            handle,
        }
    }

    pub fn div_apu_tick(&mut self, mem: &mut Memory, div_apu: u8) {
        let mut ch1 = self.ch1.lock().unwrap();
        if (div_apu & 1) != 0 {
            if ch1.length_timer == 64 {
                ch1.enabled = false;
            } else {
                ch1.length_timer += 1;
            }
        }
    }

    pub fn clock_pulse(&mut self, mem: &mut Memory) {
        let mut ch1 = self.ch1.lock().unwrap();
        ch1.period_divider += 1;

        let ch1_period_hi = mem[0xFF14];
        let duty_length = mem[0xFF11];
        ch1.duty = duty_length >> 6;
        ch1.initial_length = duty_length & 0b0011_1111;

        if ch1_period_hi & BIT_7_MASK != 0 {
            ch1.enabled = true;
            ch1.length_timer = ch1.initial_length;
            ch1.set_pulse_divider(mem);
            // TODO reset envelope timer
            // TODO set to initial volume (NR12)
            // TODO do sweep things
            mem[0xFF14] &= !BIT_7_MASK;
        }

        if ch1.period_divider >= 0x7FF {
            ch1.set_pulse_divider(mem);
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
