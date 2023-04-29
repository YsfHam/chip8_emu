use sfml::{
    graphics::{
        RenderWindow, Color, RenderTarget, View, Rect
    }, 
    window::{
        Style, Event, VideoMode
    }
};
use egui_sfml::SfEgui;

pub mod main_app;
pub mod audio;

pub struct AppSpec {
    pub title: String,
    pub window_size: VideoMode,
    pub frame_rate: u32,
    pub vsync: bool
}

impl Default for AppSpec {
    fn default() -> Self {
        Self {
            title: String::from("New App"),
            window_size: (100, 100).into(),
            frame_rate: 0,
            vsync: false,
        }
    }
}

pub trait App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut RenderWindow);
}

pub fn run_app<F>(spec: AppSpec, app_creator: F)
where
    F: FnOnce(&egui::Context) -> Box<dyn App>
{
    let mut window = create_window(spec);

    let mut sfegui = SfEgui::new(&window);

    let mut app = app_creator(sfegui.context());

    while window.is_open() {
        while let Some(event) = window.poll_event() {

            sfegui.add_event(&event);
            match event {
                Event::Closed => window.close(),
                Event::Resized { width, height } => {
                    window.set_view(&View::from_rect(Rect::new(0.0, 0.0, width as f32, height as f32)));
                }
                _ => {}
            }
        }

        sfegui
            .do_frame(|ctx| {
                app.update(ctx, &mut window);
            })
            .unwrap();

        window.clear(Color::BLACK);
        sfegui.draw(&mut window, None);
        window.display();
    }
}

fn create_window(spec: AppSpec) -> RenderWindow {
    let mut window = RenderWindow::new(
        spec.window_size,
        &spec.title,
        Style::DEFAULT,
        &Default::default()
    );

    window.set_framerate_limit(spec.frame_rate);
    window.set_vertical_sync_enabled(spec.vsync);

    window
}