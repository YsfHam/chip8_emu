use sfml::window::VideoMode;

mod chip8_core;
mod app;
mod timer;

fn main() {
    let spec = app::AppSpec {
        window_size: VideoMode::desktop_mode(),
        title: String::from("Chip8 Emulator"),
        ..Default::default()
    };

    app::run_app(spec, |ctx| {
        Box::new(
            app::main_app::MainApp::new(ctx)
        )
    });
}
