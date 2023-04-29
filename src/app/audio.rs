use rodio::source::SineWave;
use rodio::{OutputStream, OutputStreamHandle, Source};
use std::time::Duration;

pub struct Chip8Buzzer {
    stream_handle: OutputStreamHandle,
    _stream: OutputStream,
    frequency: f32,
    amplitude: f32,
}

impl Chip8Buzzer {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            stream_handle,
            _stream: stream,
            frequency: 440.0,
            amplitude: 0.2,
        }
    }

    pub fn play(&self) {
        let source = SineWave::new(self.frequency)
        .amplify(self.amplitude)
        .take_duration(Duration::from_millis(16))
        ;

        self.stream_handle.play_raw(source).unwrap();
    }

    pub fn draw_control(&mut self, ui: &mut egui::Ui) {
        let freq_control = egui::DragValue::new(&mut self.frequency)
        .clamp_range(300.0..=1000.0);
        let amplitude_control = egui::DragValue::new(&mut self.amplitude)
        .clamp_range(0.2..=10.0);
        ui.horizontal(|ui|{
            ui.label("Sound Frequency");
            ui.add(freq_control);
        });


        ui.horizontal(|ui| {
            ui.label("Amplitude");
            ui.add(amplitude_control);
        });

    }
}