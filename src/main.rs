use eframe::egui;
use std::time::{Duration, Instant};

struct HaloApp {
    running: bool,
    start_time: Option<Instant>,
    elapsed: Duration,
}

impl Default for HaloApp {
    fn default() -> Self {
        Self {
            running: false,
            start_time: None,
            elapsed: Duration::from_secs(0),
        }
    }
}

impl HaloApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn format_timecode(&self) -> String {
        let total_secs = self.elapsed.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = self.elapsed.subsec_millis();

        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }
}

impl eframe::App for HaloApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update elapsed time if running
        if self.running {
            if let Some(start) = self.start_time {
                self.elapsed = start.elapsed();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Use large text for the timecode display
            let text_style = egui::TextStyle::Heading;
            let font_id = egui::FontId::proportional(72.0);

            ui.spacing_mut().item_spacing.y = 20.0;

            // Center the timecode display
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    egui::RichText::new(self.format_timecode())
                        .font(font_id)
                        .color(egui::Color32::WHITE),
                );
            });

            // Add some space before the buttons
            ui.add_space(20.0);

            // Center-align the buttons
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(if self.running { "Stop" } else { "Start" })
                        .clicked()
                    {
                        self.running = !self.running;
                        if self.running {
                            // calculate the start time from now substract the elapsed time
                            // this way we can continue from where we left off
                            // instead of starting from 0
                            // self.start_time = Some(Instant::now());
                            // self.elapsed = Duration::from_secs(0);

                            self.start_time = Some(Instant::now() - self.elapsed);
                        }
                    }

                    if ui.button("Reset").clicked() {
                        self.elapsed = Duration::from_secs(0);
                        if self.running {
                            self.start_time = Some(Instant::now());
                        }
                    }
                });
            });
        });

        // Request continuous repaint while running
        if self.running {
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        // initial_window_size: Some(egui::vec2(400.0, 200.0)),
        // min_window_size: Some(egui::vec2(300.0, 150.0)),
        viewport: eframe::egui::ViewportBuilder {
            title: Some(String::from("Halo")),
            app_id: Some(String::from("io.github.robmorgan.halo")),
            ..eframe::egui::ViewportBuilder::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Halo",
        native_options,
        Box::new(|cc| Ok(Box::new(HaloApp::new(cc)))),
    )
}
