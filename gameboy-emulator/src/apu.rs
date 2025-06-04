use crate::audio_registers::{AudioRegisters, InternalAudioRegisters};
use crate::memory::Memory;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const BIT_6_MASK: u8 = 1 << 6;
const BIT_7_MASK: u8 = 1 << 7;

fn read_period(regs: &InternalAudioRegisters, reg_hi: u16, reg_lo: u16) -> u16 {
    let ch1_period_lo = regs[reg_lo];
    let ch1_period_hi = regs[reg_hi];
    (((ch1_period_hi & 0b111) as u16) << 8) | (ch1_period_lo as u16)
}

pub trait Sweep {
    fn write_period(&mut self, regs: &mut InternalAudioRegisters, period: u16) -> bool {
        let period_lo = (period & 0xFF) as u8;
        let period_hi = (period >> 8) as u8 & 0b111;
        regs[0xFF13] = period_lo;
        regs[0xFF14] = (regs[0xFF14] & 0b1111_1000) | period_hi;
        !(period > 0x7FF)
    }
    fn read_sweep(&mut self, mem: &InternalAudioRegisters);
    fn sweep_iteration(
        &mut self,
        mem: &mut InternalAudioRegisters,
        reg_hi: u16,
        reg_lo: u16,
    ) -> bool;
}

struct WithSweep {
    pace: u8,
    direction: u8,
    step_pow: u16,
}

impl Sweep for WithSweep {
    fn read_sweep(&mut self, regs: &InternalAudioRegisters) {
        let byte = regs[0xFF10];
        self.pace = (byte >> 4) & 0b111;
        self.direction = (byte >> 3) & 1;
        let step = byte & 0b111;
        self.step_pow = 1 << step;
    }

    fn sweep_iteration(
        &mut self,
        regs: &mut InternalAudioRegisters,
        reg_hi: u16,
        reg_lo: u16,
    ) -> bool {
        let period = read_period(regs, reg_hi, reg_lo);
        let new_period = if self.direction == 0 {
            period + period / self.step_pow
        } else {
            period - period / self.step_pow
        };
        self.write_period(regs, new_period)
    }
}

struct WithoutSweep {}

impl Sweep for WithoutSweep {
    fn read_sweep(&mut self, _: &InternalAudioRegisters) {
        // No-op
    }

    fn sweep_iteration(&mut self, _: &mut InternalAudioRegisters, _: u16, _: u16) -> bool {
        // No-op
        true
    }
}

struct PulseChannel<S: Sweep> {
    channel_num: u8,
    reg_length_duty: u16,
    reg_volume_envelope: u16,
    reg_period_lo: u16,
    reg_period_hi: u16,
    enabled: bool,
    freq: f32,
    duty: u8,
    initial_length: u8,
    length_timer: u8,
    period_divider: u16,
    sweep: S,
}

impl<S: Sweep> PulseChannel<S> {
    pub fn new(
        sweep: S,
        channel_num: u8,
        length_duty: u16,
        volume_envelope: u16,
        period_hi: u16,
        period_lo: u16,
    ) -> PulseChannel<S> {
        Self {
            channel_num,
            reg_length_duty: length_duty,
            reg_volume_envelope: volume_envelope,
            reg_period_hi: period_hi,
            reg_period_lo: period_lo,
            enabled: false,
            freq: 0.0,
            duty: 0,
            initial_length: 0,
            length_timer: 0,
            period_divider: 0,
            sweep,
        }
    }

    pub fn set_pulse_divider(&mut self, regs: &InternalAudioRegisters) {
        let period = read_period(regs, self.reg_period_hi, self.reg_period_lo);
        self.period_divider = period;
        self.freq = (131072 / (2048 - period as u32)) as f32;
    }

    pub fn div_apu_tick(&mut self, mem: &mut Memory, div_apu: u8) {
        if div_apu % 4 == 0 {
            let sweep_enabled = self.sweep.sweep_iteration(
                &mut mem.audio.internal,
                self.reg_period_hi,
                self.reg_period_lo,
            );
            let channel_enabled = self.get_channel_enabled(&mem.audio.internal);
            self.set_channel_enabled(&mut mem.audio.internal, channel_enabled & sweep_enabled)
        }

        if (div_apu & 1) == 0 && self.length_enabled(&mem.audio.internal) {
            if self.length_timer == 64 {
                self.set_channel_enabled(&mut mem.audio.internal, false);
                println!("lt a{} {} {:08b} {:08b}", self.channel_num, self.length_timer, mem.audio.internal[0xFF26], mem.audio.internal[0xFF14]);
                self.set_length_enabled(&mut mem.audio.internal, false);
            } else {
                self.length_timer += 1;
            }
            println!("lt{} {} {:08b} {:08b}", self.channel_num, self.length_timer, mem.audio.internal[0xFF26], mem.audio.internal[0xFF14]);
        }
    }

    fn get_channel_enabled(&mut self, regs: &InternalAudioRegisters) -> bool {
        let value = ((regs[0xFF26] >> self.channel_num) & 1) > 0;
        self.enabled = value;
        value
    }

    fn set_channel_enabled(&mut self, regs: &mut InternalAudioRegisters, enabled: bool) {
        if enabled {
            regs[0xFF26] |= 1 << self.channel_num;
            if !self.enabled {
                println!("Trigger {:08b}", regs[0xFF26]);
            }
        } else {
            if self.enabled {
                println!("Cleared");
            }
            regs[0xFF26] &= !(1 << self.channel_num);
        }
        self.enabled = enabled;
    }

    pub fn clock_pulse(&mut self, mem: &mut Memory) {
        self.period_divider += 1;

        let ch1_period_hi = mem.audio.internal[self.reg_period_hi];
        let duty_length = mem.audio.internal[self.reg_length_duty];
        self.duty = duty_length >> 6;
        self.initial_length = duty_length & 0b0011_1111;

        if ch1_period_hi & BIT_6_MASK != 0 {
            self.set_length_enabled(&mut mem.audio.internal, true);
        }

        // Trigger event
        if ch1_period_hi & BIT_7_MASK != 0 {
            println!("Trigger a {:08b}", ch1_period_hi);
            self.set_channel_enabled(&mut mem.audio.internal, true);
            self.length_timer = self.initial_length;
            self.set_pulse_divider(&mut mem.audio.internal);
            // TODO reset envelope timer
            // TODO set to initial volume (NR12)
            // TODO do sweep things
            self.sweep.read_sweep(&mem.audio.internal);
            mem.audio.internal[self.reg_period_hi] &= !BIT_7_MASK;
        }

        if self.period_divider >= 0x7FF {
            self.set_pulse_divider(&mut mem.audio.internal);
        }
    }

    fn length_enabled(&self, regs: &InternalAudioRegisters) -> bool {
        (regs[self.reg_period_hi] >> 6) & 1 > 0
    }

    fn set_length_enabled(&self, regs: &mut InternalAudioRegisters, value: bool) {
        let mask = 1 << 6;
        regs[self.reg_period_hi] = (regs[self.reg_period_hi] & !mask) | (value as u8 * mask);
    }
}

pub struct APU {
    wave_period_divider: u16,
    ch1: Arc<Mutex<PulseChannel<WithSweep>>>,
    ch2: Arc<Mutex<PulseChannel<WithoutSweep>>>,
    handle: thread::JoinHandle<()>,
}

impl APU {
    pub fn new() -> Self {
        let ch1 = Arc::new(Mutex::new(PulseChannel::new(
            WithSweep {
                pace: 0,
                direction: 0,
                step_pow: 1,
            },
            0,
            0xFF11,
            0xFF12,
            0xFF14,
            0xFF13,
        )));
        let ch2 = Arc::new(Mutex::new(PulseChannel::new(
            WithoutSweep {},
            1,
            0xFF16,
            0xFF17,
            0xFF19,
            0xFF18,
        )));

        let ch1_clone = ch1.clone();
        let ch2_clone = ch2.clone();
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
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
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
                            let mut c = ch2.lock().unwrap();
                            if !c.enabled {
                                for sample in data.iter_mut() {
                                    *sample += 0.0;
                                }
                            } else {
                                let bound = 1.0 / (2 * c.duty as i8 - 2) as f32;
                                for sample in data.iter_mut() {
                                    sample_clock = (sample_clock + 1.0) % sample_rate;
                                    let amp = sample_clock * c.freq * 2.0 * std::f32::consts::PI
                                        / sample_rate;
                                    *sample += if amp.sin() > bound { 0.5 } else { -0.5 };
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
            ch2: ch2_clone,
            handle,
        }
    }

    pub fn div_apu_tick(&mut self, mem: &mut Memory, div_apu: u8) {
        if !self.apu_enabled(&mem.audio.internal) {
            // TODO disable audio players as well
            return;
        }
        let mut ch1 = self.ch1.lock().unwrap();
        ch1.div_apu_tick(mem, div_apu);

        let mut ch2 = self.ch2.lock().unwrap();
        ch2.div_apu_tick(mem, div_apu);
    }

    pub fn clock_pulse(&mut self, mem: &mut Memory) {
        if !self.apu_enabled(&mem.audio.internal) {
            return;
        }
        let mut ch1 = self.ch1.lock().unwrap();
        ch1.clock_pulse(mem);
    }

    pub fn clock_wave(&mut self, mem: &mut Memory) {
        if !self.apu_enabled(&mem.audio.internal) {
            return;
        }
        self.wave_period_divider += 1;
    }

    fn apu_enabled(&self, mem: &InternalAudioRegisters) -> bool {
        mem[0xFF26] & (1 << 7) == 0
    }

    pub fn update(&mut self, mem: &mut Memory) {
        let master_ctrl = mem[0xFF26];
        let panning = mem[0xFF25];
        let volume_vin = mem[0xFF24];

        let on_off = (master_ctrl & BIT_7_MASK) != 0;

        if mem[0xFF10] & 0b10000000 != 0b10000000 {
            mem[0xFF10] |= 0b10000000;
        }
        if mem[0xFF14] & 0b00111000 != 0b00111000 {
            mem[0xFF14] |= 0b00111000;
        }
        if mem[0xFF19] & 0b00111000 != 0b00111000 {
            mem[0xFF19] |= 0b00111000;
        }
        if mem[0xFF1A] & 0b01111111 != 0b01111111 {
            mem[0xFF1A] |= 0b01111111;
        }
        if mem[0xFF1C] & 0b10011111 != 0b10011111 {
            mem[0xFF1C] |= 0b10011111;
        }
        if mem[0xFF1C] & 0b10011111 != 0b10011111 {
            mem[0xFF1C] |= 0b10011111;
        }
        if mem[0xFF1E] & 0b00111000 != 0b00111000 {
            mem[0xFF1E] |= 0b00111000;
        }
        if mem[0xFF20] & 0b11000000 != 0b11000000 {
            mem[0xFF20] |= 0b11000000;
        }
        if mem[0xFF23] & 0b00111111 != 0b00111111 {
            mem[0xFF23] |= 0b00111111;
        }
        if mem[0xFF26] & 0b01110000 != 0b01110000 {
            mem[0xFF26] |= 0b01110000;
        }
    }
}
