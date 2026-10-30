use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use amstrad_cpc::dsk::DiskImage;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
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

    shift_held: [bool; 2],
    ctrl_held: [bool; 2],

    // The menu must stay alive as long as the window
    menu: Option<Menu>,
    insert_disk_item: MenuItem,
    eject_disk_item: MenuItem,
    reset_item: MenuItem,
    exit_item: MenuItem,
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
            shift_held: [false; 2],
            ctrl_held: [false; 2],
            menu: None,
            insert_disk_item: MenuItem::new("&Insert disk...", true, None),
            eject_disk_item: MenuItem::new("&Eject disk", true, None),
            reset_item: MenuItem::new("&Reset", true, None),
            exit_item: MenuItem::new("E&xit", true, None),
        }
    }

    fn run_one_frame(&mut self) {
        let mut cycles_this_frame = self.cycle_remainder;

        while cycles_this_frame < CYCLES_PER_FRAME {
            self.cpu.n_int = !self.amstrad.gate_array.interrupt_pending;

            let raw_cycles = self.cpu.execute(&mut self.amstrad);

            // The request stays asserted until the CPU really takes the interrupt (INTA)
            if self.cpu.int_acknowledged {
                self.amstrad.gate_array.acknowledge_interrupt();
                self.cpu.n_int = true;
            }

            // Alignment over 4 T-states (1 µs = 1 cycle Gate Array / CRTC)
            let aligned_cycles = ((raw_cycles + 3) & !3) as u32;
            cycles_this_frame += aligned_cycles;
            self.audio_cycle_accumulator += aligned_cycles as f32;

            while self.audio_cycle_accumulator >= self.cycles_per_sample {
                self.audio_cycle_accumulator -= self.cycles_per_sample;

                let sample = self.amstrad.psg.generate_sample();

                if self.audio.producer.free_len() >= self.audio.channels {
                    for _ in 0..self.audio.channels {
                        let _ = self.audio.producer.push(sample);
                    }
                }
            }

            let video_steps = aligned_cycles / 4;
            for _ in 0..video_steps {
                self.amstrad.step_video();
            }
        }

        // Save the overflow (0, 4, 8... T-states) to keep a perfect timing
        self.cycle_remainder = cycles_this_frame - CYCLES_PER_FRAME;
    }

    fn build_menu(&self) -> Menu {
        let menu = Menu::new();
        let file_menu = Submenu::new("&File", true);
        file_menu.append(&self.insert_disk_item).unwrap();
        file_menu.append(&self.eject_disk_item).unwrap();
        file_menu.append(&PredefinedMenuItem::separator()).unwrap();
        file_menu.append(&self.reset_item).unwrap();
        file_menu.append(&PredefinedMenuItem::separator()).unwrap();
        file_menu.append(&self.exit_item).unwrap();
        menu.append(&file_menu).unwrap();
        menu
    }

    fn insert_disk_dialog(&mut self) {
        // The dialog is modal: the emulation is paused while it is open
        let picked = rfd::FileDialog::new()
            .set_title("Insert a disk image")
            .add_filter("Amstrad disk image", &["dsk"])
            .pick_file();

        if let Some(path) = picked {
            match path.to_str().map(DiskImage::load) {
                Some(Ok(disk)) => self.amstrad.fdc.disk = Some(disk),
                Some(Err(e)) => eprintln!("Unable to load the disk image: {e}"),
                None => eprintln!("Invalid disk image path: {}", path.display()),
            }
        }

        // Do not try to catch up the time spent in the dialog, and drop any key state
        self.next_frame_time = Instant::now() + FRAME_DURATION;
        self.shift_held = [false; 2];
        self.ctrl_held = [false; 2];
        self.amstrad.psg.keyboard_matrix = [0xFF; 10];
    }

    /// Cold reset: new CPU and new machine, the inserted disk is kept in the drive
    fn reset(&mut self) {
        let disk = self.amstrad.fdc.disk.take();

        self.cpu = create_cpu();
        self.amstrad = create_machine(self.audio.sample_rate);
        self.amstrad.fdc.disk = disk;

        self.cycle_remainder = 0;
        self.audio_cycle_accumulator = 0.0;
        self.next_frame_time = Instant::now() + FRAME_DURATION;
        self.shift_held = [false; 2];
        self.ctrl_held = [false; 2];
    }

    fn handle_menu_events(&mut self, event_loop: &ActiveEventLoop) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == *self.insert_disk_item.id() {
                self.insert_disk_dialog();
            } else if event.id == *self.eject_disk_item.id() {
                self.amstrad.fdc.disk = None;
            } else if event.id == *self.reset_item.id() {
                self.reset();
            } else if event.id == *self.exit_item.id() {
                event_loop.exit();
            }
        }
    }
}

impl ApplicationHandler for EmulatorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attrs = Window::default_attributes()
                .with_title("Amstrad CPC 6128 Emulator")
                .with_inner_size(PhysicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT))
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

            // Windows only: attach the menu bar to the native window
            #[cfg(target_os = "windows")]
            {
                use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

                let menu = self.build_menu();
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(h) = handle.as_raw() {
                        unsafe { menu.init_for_hwnd(h.hwnd.get()).unwrap() };
                    }
                }
                self.menu = Some(menu);

                // The menu bar takes some room: ask again for the full client area
                let _ = window.request_inner_size(PhysicalSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));
            }

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
                        repeat,
                        ..
                    },
                ..
            } => {
                // The CPC keyboard is scanned by the firmware: OS auto-repeat must be ignored
                if repeat {
                    return;
                }

                if keycode == KeyCode::F12 && state == ElementState::Pressed {
                    println!("--- MANUAL TRACE DUMP TRIGGERED ---");
                    self.cpu.dump_trace(&mut self.amstrad);
                }

                let is_pressed = state == ElementState::Pressed;
                match keycode {
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        self.shift_held[(keycode == KeyCode::ShiftRight) as usize] = is_pressed
                    }
                    KeyCode::ControlLeft | KeyCode::ControlRight => {
                        self.ctrl_held[(keycode == KeyCode::ControlRight) as usize] = is_pressed
                    }
                    _ => {}
                }

                if let Some((line, bit)) = map_key_to_matrix(keycode) {
                    // Shift and Ctrl share one CPC key: stay down while either physical key is held
                    let down = match keycode {
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => self.shift_held.contains(&true),
                        KeyCode::ControlLeft | KeyCode::ControlRight => {
                            self.ctrl_held.contains(&true)
                        }
                        _ => is_pressed,
                    };
                    self.amstrad.psg.set_key_state(line, bit, down);
                }
            }
            WindowEvent::Focused(false) => {
                // Key release events are lost when the focus is gone
                self.shift_held = [false; 2];
                self.ctrl_held = [false; 2];
                self.amstrad.psg.keyboard_matrix = [0xFF; 10];
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
                        // To give a CRT monitor effect, the second line is dimmed (30% dimmer)
                        for (dst_pixel, &src_pixel) in dst_odd.iter_mut().zip(src_line.iter()) {
                            *dst_pixel = src_pixel - ((src_pixel >> 2) & 0x003F3F3F);
                        }
                    }

                    buffer.present().unwrap();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_menu_events(event_loop);

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

/// Builds a powered-on CPC with its ROMs, as after a cold start (no disk inserted)
fn create_machine(sample_rate: f32) -> AmstradCpc6128 {
    let mut amstrad = AmstradCpc6128::new(sample_rate);
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
    amstrad
}

fn create_cpu() -> Z80 {
    let mut cpu = Z80::new();
    cpu.trace_enabled = true; // Needed by the F12 trace dump
    cpu
}

fn main() {
    let audio = AudioSystem::new();

    let amstrad = create_machine(audio.sample_rate);

    let cpu = create_cpu();

    let event_loop = EventLoop::new().expect("Can't init EventLoop");
    let mut app = EmulatorApp::new(cpu, amstrad, audio);

    event_loop
        .run_app(&mut app)
        .expect("Error while executing winit loop");
}
