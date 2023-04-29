use crate::chip8_core::chip8_errors::Chip8ErrorKind;
use crate::chip8_core::graphics::FrameBuffer;
use crate::chip8_core::instruction::Instruction;

use crate::chip8_core::{
    chip8::Chip8,
    cpu::CPU,
    memory::Ram,
    PROGRAM_START_ADDRESS,
    graphics::{SCREEN_HEIGHT, SCREEN_WIDTH},
    keypad::CHIP8_KEYPAD,
};
use crate::timer::Timer;
use super::audio::Chip8Buzzer;

struct ScreenFramebuffer {
    texture: egui::TextureHandle,

    background_color: [u8; 3],
    foreground_color: [u8; 3],
}

impl ScreenFramebuffer {
    fn new(ctx: &egui::Context) -> Self {

        let options = egui::TextureOptions::NEAREST;

        let texture = {
            ctx.load_texture("ScreenTex", 
            egui::ColorImage::from_rgb([SCREEN_WIDTH, SCREEN_HEIGHT], &[0; SCREEN_HEIGHT * SCREEN_WIDTH * 3]), 
            options)
        };

        Self {
            texture,
            background_color: [0; 3],
            foreground_color: [255; 3],
        }
    }
}

impl FrameBuffer for ScreenFramebuffer {
    fn update(&mut self, screen: &crate::chip8_core::graphics::Screen) {
            let mut image_buffer = Vec::new();
            for y in 0..SCREEN_HEIGHT {
                for x in 0..SCREEN_WIDTH {
                    if screen.is_pixel_set(x, y) {
                        image_buffer.extend_from_slice(&self.foreground_color);
                    }
                    else {
                        image_buffer.extend_from_slice(&self.background_color);
    
                    }
                }
            }
    
            self.texture.set(
                egui::ColorImage::from_rgb([SCREEN_WIDTH, SCREEN_HEIGHT], image_buffer.as_slice()),
                egui::TextureOptions::NEAREST
            );
    }
}

fn draw_cpu(ui: &mut egui::Ui, cpu: &CPU) {
    ui.vertical_centered(|ui| {
        ui.heading("CPU");
    });
    ui.separator();
    let space = 20.0;
    ui.label(format!("PC = {:#X}", cpu.pc));
    ui.add_space(space);

    ui.label(format!("I = {:#X}", cpu.i));
    ui.add_space(space);

    ui.label(format!("SP = {:#X}", cpu.stack.sp));
    ui.add_space(space);

    ui.label(format!("Sound timer = {}", cpu.sound_timer));
    ui.add_space(space);

    ui.label(format!("Delay timer = {}", cpu.delay_timer));
    ui.add_space(space);

    egui::Grid::new("registers_display")
    .spacing((5.0, 10.0))
    .show(ui, |ui| {
        for i in 0..4 {
            for j in 0..4 {
                let index = i * 4 + j;
                ui.label(format!("V{:X} = {:#04X}", index, cpu.v[index]));
            }
            ui.end_row();
        }
    });
}

fn draw_code(ui: &mut egui::Ui, ram: &Ram, start: usize, length: usize, pc: usize) {
    let code = ram.read_bytes(start as u16, (PROGRAM_START_ADDRESS + length) as u16)
    .unwrap();

    ui.vertical_centered(|ui| {
        ui.heading("Program");
    });
    ui.separator();

    let make_text = |change_color: bool, color: egui::Color32, text_str: &str| {
        let mut text = egui::RichText::new(text_str);
        if change_color {
            text = text.color(color);
        }
        text
    };

    ui.group(|ui| {
        egui::ScrollArea::new([false, true])
        .scroll2([false, true])
        .show_rows(ui, 0.0, (ui.available_height() / ui.spacing().combo_height) as usize, |ui, _| {
            let mut i = 0;
            while i < code.len() {
                let hi = (code[i] as u16) << 8;
                i += 1;
                if i >= code.len() {
                    break;
                }
                let lo = code[i] as u16;
                i += 1;
                
                let opcode = hi | lo;
                ui.vertical_centered(|ui| {
                    let addr = start + i - 2;
                    let change_color = addr == pc;
                    let text = format!("{:#04X}\t{}", addr, Instruction::new(opcode).to_string());
                    ui.label(make_text(change_color, egui::Color32::GREEN, &text));
                });
                if i < code.len() - 1 {
                    ui.separator();
                }
            }
        });
    });
    
}

#[derive(PartialEq)]
enum ExecutionState {
    Pause,
    Continue,
    RunNext,
    Halt,
}

struct GameLoadingWindow {
    error: std::io::Error,
    loaded_file_name: String,
}

const MAP_KEYS_LAYOUT: [egui::Key; 16] = [
    egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4,
    egui::Key::A, egui::Key::Z, egui::Key::E, egui::Key::R,
    egui::Key::Q, egui::Key::S, egui::Key::D, egui::Key::F,
    egui::Key::W, egui::Key::X, egui::Key::C, egui::Key::V,
];

pub struct MainApp {
    chip8: Chip8,

    game_loading_window: Option<GameLoadingWindow>,
    update_timer: Timer,
    framebuffer: ScreenFramebuffer,

    exec_state: ExecutionState,
    game_freq: f64,

    debug_mode: bool,

    show_settings_window: bool,

    buzzer: Chip8Buzzer,

    run_program_result: Option<Chip8ErrorKind>
}

impl MainApp {
    pub fn new(ctx: &egui::Context) -> Self {

        Self {
            chip8: Chip8::new(),
            game_loading_window: None,
            update_timer: Timer::new(),
            framebuffer: ScreenFramebuffer::new(ctx),
            #[cfg(debug_assertions)]
            exec_state: ExecutionState::Pause,
            #[cfg(not(debug_assertions))]
            exec_state: ExecutionState::Continue,
            game_freq: 500.0,
            #[cfg(debug_assertions)]
            debug_mode: true,
            #[cfg(not(debug_assertions))]
            debug_mode: false,
            show_settings_window: false,
            buzzer: Chip8Buzzer::new(),
            run_program_result: None
        }
    }

    fn cpu_info_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("cpu_info")
        .min_width(300.0)
        .resizable(false)
        .show(ctx, |ui| {
            draw_cpu(ui, self.chip8.get_cpu());
            ui.separator();
            let prog_length = self.chip8.get_program_length();

            let pc = self.chip8.get_cpu().pc as usize;

            let start = if pc > PROGRAM_START_ADDRESS {
                let diff = pc - PROGRAM_START_ADDRESS;
                pc - std::cmp::min(diff, 6)
            }
            else {
                PROGRAM_START_ADDRESS
            };
            if prog_length > 0 {
                draw_code(ui, self.chip8.get_ram(), start, prog_length, pc);
            }
            else {
                ui.heading("No program loaded, use the menu Game to load one");
            }
        });
    }

    fn reset(&mut self) {
        self.chip8.reset();
        if self.debug_mode {
            self.exec_state = ExecutionState::Pause;
        }
    }

    fn game_menu(&mut self, ui: &mut egui::Ui, frame: &mut sfml::graphics::RenderWindow) -> bool {
        if ui.button("Load new Game").clicked() {
            
            let file = rfd::FileDialog::new().pick_file();
            if let Some(file) = file {
               
                let file_name = file.file_name().unwrap().to_string_lossy().to_string();
                self.reset();

                match self.chip8.load_program(&file) {
                    Ok(_) => {
                        frame.set_title(&file_name);
                    },
                    Err(e) => {
                        self.game_loading_window = Some(GameLoadingWindow { error: e, loaded_file_name: file_name });
                    }
                }
            }
            true
        }
        else if ui.button("Reload").clicked() {
            self.chip8.reload_program();
            true
        }
        else if ui.button("Settings").clicked() {
            self.show_settings_window = true;
            true
        }
        else {
            false
        }
    }

    fn run_program(&mut self) {
        let can_run = 
        self.chip8.get_program_length() > 0 &&
        self.chip8.get_program_length() + PROGRAM_START_ADDRESS - 2 >= self.chip8.get_cpu().pc as usize;

        if can_run {
            match self.chip8.run_instruction() {
                Ok(_) => {},
                Err(e) => {
                    self.exec_state = ExecutionState::Halt;
                    self.run_program_result = Some(e);
                    self.debug_mode = true;
                }
            }
        }
    }

    fn draw_screen(&mut self, ui: &mut egui::Ui) {

        self.chip8.update_framebuffer(&mut self.framebuffer);

        let width = ui.available_width();

        let tex_size = self.framebuffer.texture.size_vec2();
        let aspect_ratio = tex_size.x / tex_size.y;
        let height = width / aspect_ratio;

        ui.image(&self.framebuffer.texture, egui::vec2(width, height));
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        let keyboard = self.chip8.get_keypad_mut();
        for key_index in 0..16 {
            ctx.input(|i| {
                if i.key_pressed(MAP_KEYS_LAYOUT[key_index]) {
                    keyboard.set_key_pressed(CHIP8_KEYPAD[key_index], true);
                }
                else if i.key_released(MAP_KEYS_LAYOUT[key_index]) {
                    keyboard.set_key_pressed(CHIP8_KEYPAD[key_index], false);
                }
            })
        }
    }

    fn draw_keypad(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("KeyPad");
        });

        ui.separator();
        egui::Grid::new("KeyPad")
        .show(ui, |ui| {
            for y in 0..4 {

                for x in 0..4 {
                    let index = y * 4 + x;
                    let mut text = egui::RichText::new(&format!("{:?} ---> {:?}", MAP_KEYS_LAYOUT[index], CHIP8_KEYPAD[index]));
                    if self.chip8.get_keypad().key_pressed(CHIP8_KEYPAD[index]) {
                        text = text.color(egui::Color32::GREEN);
                    }
                    ui.label(text);
                }
                ui.end_row();
            }
        });
    }

    fn draw_debug_control(&mut self, ui: &mut egui::Ui) {
        match self.exec_state {
            ExecutionState::Pause => {
                if ui.button("Next").clicked() {
                    self.exec_state = ExecutionState::RunNext;
                }

                if ui.button("Continue").clicked() {
                    self.exec_state = ExecutionState::Continue;
                }
            },
            ExecutionState::Continue => {
                if ui.button("Pause").clicked() {
                    self.exec_state = ExecutionState::Pause;
                }
            }
            ExecutionState::Halt => {
                if ui.button("Reload").clicked() {
                    self.chip8.reload_program();
                    self.exec_state = ExecutionState::Pause;
                }
            },
            _ => {}
        }
        
    }
}

impl super::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut sfml::graphics::RenderWindow) {
        let delta_time = self.update_timer.restart().as_secs_f64();

        let nb_cycles = (self.game_freq * delta_time).round() as u32;

        self.handle_input(ctx);
        if self.chip8.can_play_sound() {
            self.buzzer.play();
        }
        match self.exec_state {
            ExecutionState::Continue => {
                for _ in 0..nb_cycles{
                    self.run_program();
                }
            },
            ExecutionState::RunNext => {
                self.run_program();
                self.exec_state = ExecutionState::Pause;
            },
            _ => {},

        }

        // Rendering
        let mut close_exec_error_window = false;
        if let Some(e) = &self.run_program_result {
            egui::Window::new("Execution error")
            .pivot(egui::Align2::CENTER_TOP)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, -5.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(format!("{}", e));
                    if ui.button("Ok").clicked() {
                        close_exec_error_window = true;
                    }
                });
            });
        }

        if close_exec_error_window {
            self.run_program_result = None;
        }

        if self.debug_mode {
            self.cpu_info_panel(ctx);
        }

        egui::TopBottomPanel::top("menu_panel")
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Game", |ui| {
                    if self.game_menu(ui, frame) {
                        ui.close_menu();
                    }
                });
                ui.menu_button("Mode", |ui| {
                    if ui.button("Debug mode").clicked() {
                        self.debug_mode = true;
                        ui.close_menu();
                    }
                    if ui.button("Normal mode").clicked() {
                        self.debug_mode = false;
                        self.exec_state = ExecutionState::Continue;
                        ui.close_menu();
                    };
                });
            });
        });

        if self.chip8.get_program_length() > 0 && self.debug_mode {
            egui::TopBottomPanel::top("debug_control")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let space = ui.available_width();
                    ui.add_space(space / 2.0);
                    self.draw_debug_control(ui);
                });
            });
        }

        egui::CentralPanel::default()
        .show(ctx, |ui| {
            self.draw_screen(ui);
            if self.debug_mode {
                //ui.add_space(30.0);
                egui::SidePanel::left("keypad").show_inside(ui, |ui| {
                    ui.group(|ui| {
                        self.draw_keypad(ui);
                    });
                });
            }
        });
        if let Some(game_loading_window_error) = &self.game_loading_window {
            let close = show_game_loading_error_window(ctx, game_loading_window_error);
            if close {
                self.game_loading_window = None;
            }
        }
        if self.show_settings_window {
            show_settings_window(ctx, self);
        }

        
    }
}

fn show_settings_window(ctx: &egui::Context, app: &mut MainApp) {
    egui::Window::new("Settings")
    .show(ctx, |ui| {
        let drag_value = egui::DragValue::new(&mut app.game_freq)
        .clamp_range(100.0..=1000.0);
        ui.horizontal(|ui| {
            ui.label("CPU frequency");
            ui.add(drag_value);
            ui.label("Hz");
        });
        ui.group(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Screen colors");
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Background color");
                if ui.color_edit_button_srgb(&mut app.framebuffer.background_color).changed() {
                    app.framebuffer.update(app.chip8.get_screen());
                }
    
                ui.label("Foreground color");
                if ui.color_edit_button_srgb(&mut app.framebuffer.foreground_color).changed() {
                    app.framebuffer.update(app.chip8.get_screen());
                }
            });
        });
        ui.group(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Sound control");
            });
            ui.separator();
            app.buzzer.draw_control(ui);
        });
        if !app.debug_mode {
            ui.group(|ui| {
                app.draw_keypad(ui);
            });
        }
        ui.vertical_centered(|ui| {
            if ui.button("Close").clicked() {
                app.show_settings_window = false;
            }
        });
    });
}

fn show_game_loading_error_window(ctx: &egui::Context, game_loading_window_error: &GameLoadingWindow) -> bool {
    let mut close = false;
    egui::Window::new("Game loading error")
    .resizable(false)
    .collapsible(false)
    .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
    .show(ctx, |ui| {
        let GameLoadingWindow { error, loaded_file_name } = game_loading_window_error;
        ui.label(format!("Error while loading {}: {:?}", loaded_file_name, error.kind()));
        ui.vertical_centered(|ui| {
            if ui.button("Ok").clicked() {
                close = true;
            }
        });
    });

    close
}