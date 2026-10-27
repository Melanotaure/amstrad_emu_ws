use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use amstrad_cpc::dsk::DiskImage;
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use amstrad_cpc::amstrad::AmstradCpc6128;
use amstrad_cpc::audio::AudioSystem;
use amstrad_cpc::keyboard::map_key_to_matrix;
use amstrad_cpc::video::{SCREEN_HEIGHT, SCREEN_WIDTH};
use z80_core::z80::*;

const CYCLES_PER_FRAME: u32 = 79_872; // 4 MHz / 50 Hz (Image Rate for PAL standard)
const FRAME_DURATION: Duration = Duration::from_millis(20); // 1000 ms / 50 fps = 20 ms

const DISPLAY_WIDTH: u32 = SCREEN_WIDTH as u32;
const DISPLAY_HEIGHT: u32 = (SCREEN_HEIGHT * 2) as u32; // 576 pixels

struct EmulatorApp {
    cpu: Z80,
    amstrad: AmstradCpc6128,

    next_frame_time: Instant,
    cycle_remainder: u32,

    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,

    audio: AudioSystem,
    audio_cycle_accumulator: f32,
    cycles_per_sample: f32,
}

impl EmulatorApp {
    fn new(cpu: Z80, amstrad: AmstradCpc6128, audio: AudioSystem) -> Self {
        let cycles_per_sample = 4_000_000.0 / audio.sample_rate;

        Self {
            cpu,
            amstrad,
            next_frame_time: Instant::now() + FRAME_DURATION,
            cycle_remainder: 0,
            window: None,
            surface: None,
            audio,
            audio_cycle_accumulator: 0.0,
            cycles_per_sample,
        }
    }

    fn run_one_frame(&mut self) {
        // On démarre avec les éventuels cycles de débordement de la trame précédente
        let mut cycles_this_frame = self.cycle_remainder;

        while cycles_this_frame < CYCLES_PER_FRAME {
            self.cpu.n_int = !self.amstrad.gate_array.interrupt_pending;

            // 1. Exécution d'une instruction Z80
            let raw_cycles = self.cpu.execute(&mut self.amstrad);

            // Acquittement matériel si le Z80 vient de prendre l'interruption
            if !self.cpu.n_int && !self.cpu.iff1 {
                self.amstrad.gate_array.acknowledge_interrupt();
                self.cpu.n_int = true;
            }

            // Alignement sur 4 T-states (1 µs = 1 cycle Gate Array / CRTC)
            let aligned_cycles = ((raw_cycles + 3) & !3) as u32;
            cycles_this_frame += aligned_cycles;
            self.audio_cycle_accumulator += aligned_cycles as f32;

            while self.audio_cycle_accumulator >= self.cycles_per_sample {
                self.audio_cycle_accumulator -= self.cycles_per_sample;

                let sample = self.amstrad.psg.generate_sample();

                if self.audio.producer.free_len() >= 2 {
                    let _ = self.audio.producer.push(sample);
                    let _ = self.audio.producer.push(sample);
                }
            }

            // 2. Avancement de la vidéo d'autant de microsecondes
            let video_steps = aligned_cycles / 4;
            for _ in 0..video_steps {
                self.amstrad.step_video();
            }
        }

        // Sauvegarde du surplus (0, 4, 8... T-states) pour garder un timing parfait
        self.cycle_remainder = cycles_this_frame - CYCLES_PER_FRAME;
        // self.audio_cycle_accumulator = cycles_since_last_sample;
    }
}

impl ApplicationHandler for EmulatorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attrs = Window::default_attributes()
                .with_title("Amstrad CPC 6128 Emulator")
                .with_inner_size(LogicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT))
                .with_resizable(false);

            let window = Rc::new(
                event_loop
                    .create_window(win_attrs)
                    .expect("Window creation error"),
            );
            let context = Context::new(window.clone()).expect("softbuffer context error");
            let mut surface =
                Surface::new(&context, window.clone()).expect("softbuffer surface error");

            surface
                .resize(
                    NonZeroU32::new(DISPLAY_WIDTH).unwrap(),
                    NonZeroU32::new(DISPLAY_HEIGHT).unwrap(),
                )
                .unwrap();

            self.window = Some(window);
            self.surface = Some(surface);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                if keycode == KeyCode::F12 && state == ElementState::Pressed {
                    println!("--- MANUAL TRACE DUMP TRIGGERED ---");
                    self.cpu.dump_trace(&mut self.amstrad);
                }

                if let Some((line, bit)) = map_key_to_matrix(keycode) {
                    let is_pressed = state == ElementState::Pressed;
                    self.amstrad.psg.set_key_state(line, bit, is_pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(surface) = &mut self.surface {
                    let mut buffer = surface.buffer_mut().unwrap();

                    for (src_line, dst_pair) in self
                        .amstrad
                        .framebuffer
                        .pixels
                        .chunks_exact(SCREEN_WIDTH)
                        .zip(buffer.chunks_exact_mut(SCREEN_WIDTH * 2))
                    {
                        let (dst_even, dst_odd) = dst_pair.split_at_mut(SCREEN_WIDTH);

                        dst_even.copy_from_slice(src_line);
                        //dst_odd.copy_from_slice(src_line);

                        for (dst_pixel, &src_pixel) in dst_odd.iter_mut().zip(src_line.iter()) {
                            *dst_pixel = src_pixel - ((src_pixel >> 2) & 0x003F3F3F);
                        }
                    }

                    // Ultra-fast direct copy of our Vec<u32> to the window
                    // buffer.copy_from_slice(&self.amstrad.framebuffer.pixels);

                    buffer.present().unwrap();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now >= self.next_frame_time {
            self.run_one_frame();

            if let Some(window) = &self.window {
                window.request_redraw();
            }

            self.next_frame_time += FRAME_DURATION;

            if now > self.next_frame_time {
                self.next_frame_time = now + FRAME_DURATION;
            }
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }
}

fn main() {
    let audio = AudioSystem::new();

    let mut amstrad = AmstradCpc6128::new(audio.sample_rate);
    amstrad
        .memory
        .load_lower_rom("amstrad_cpc/roms/os6128.rom")
        .expect("OS ROM not found");
    amstrad
        .memory
        .load_upper_rom(0, "amstrad_cpc/roms/basic6128.rom")
        .expect("BASIC ROM not found");
    amstrad
        .memory
        .load_upper_rom(7, "amstrad_cpc/roms/amsdos.rom")
        .expect("AMSDOS ROM not found");

    amstrad.fdc.disk = DiskImage::load("amstrad_cpc/roms/FruityFrank.dsk").ok();

    let cpu = Z80::new();

    let event_loop = EventLoop::new().expect("Can't init EventLoop");
    let mut app = EmulatorApp::new(cpu, amstrad, audio);

    event_loop
        .run_app(&mut app)
        .expect("Error while executing winit loop");
}
