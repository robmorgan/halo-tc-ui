use eframe::egui;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Cue {
    name: String,
    start_time: Duration,
    duration: Duration,
    is_playing: bool,
    progress: f32,
}

impl Cue {
    fn new(name: &str, start_time_secs: u64, duration_secs: u64) -> Self {
        Self {
            name: name.to_string(),
            start_time: Duration::from_secs(start_time_secs),
            duration: Duration::from_secs(duration_secs),
            is_playing: false,
            progress: 0.0,
        }
    }

    fn update(&mut self, current_time: Duration) {
        if current_time >= self.start_time {
            let elapsed_in_cue = current_time - self.start_time;
            if elapsed_in_cue <= self.duration {
                self.is_playing = true;
                self.progress = elapsed_in_cue.as_secs_f32() / self.duration.as_secs_f32();
            } else {
                self.is_playing = false;
                self.progress = 1.0;
            }
        } else {
            self.is_playing = false;
            self.progress = 0.0;
        }
    }
}

struct HaloApp {
    running: bool,
    start_time: Option<Instant>,
    elapsed: Duration,
    cues: Vec<Cue>,
    link_enabled: bool,
    bpm: f32,
}

impl Default for HaloApp {
    fn default() -> Self {
        Self {
            running: false,
            start_time: None,
            elapsed: Duration::from_secs(0),
            cues: vec![
                Cue::new("Opening", 2, 5),
                Cue::new("First Verse", 8, 10),
                Cue::new("Chorus", 19, 8),
                Cue::new("Bridge", 28, 12),
                Cue::new("Finale", 41, 6),
            ],
            link_enabled: false,
            bpm: 120.0,
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

    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

impl eframe::App for HaloApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update elapsed time if running
        if self.running {
            if let Some(start) = self.start_time {
                self.elapsed = start.elapsed();
                // Update all cues
                for cue in &mut self.cues {
                    cue.update(self.elapsed);
                }
            }
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if self.link_enabled {
                        "Link ●"
                    } else {
                        "Link ○"
                    })
                    .clicked()
                {
                    self.link_enabled = !self.link_enabled;
                    // Here you would add the actual Ableton Link connection logic
                }
                ui.add_space(8.0);
                ui.label("BPM:");
                ui.add(
                    egui::DragValue::new(&mut self.bpm)
                        .speed(0.1)
                        .range(20.0..=300.0)
                        .fixed_decimals(1),
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Use large text for the timecode display
            let font_id = egui::FontId::proportional(72.0);

            ui.spacing_mut().item_spacing.y = 20.0;

            // Center the timecode display
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
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
                            self.start_time = Some(Instant::now() - self.elapsed);
                        }
                    }

                    if ui.button("Reset").clicked() {
                        self.elapsed = Duration::from_secs(0);
                        if self.running {
                            self.start_time = Some(Instant::now());
                        }
                        // Reset all cues
                        for cue in &mut self.cues {
                            cue.is_playing = false;
                            cue.progress = 0.0;
                        }
                    }
                });
            });

            ui.add_space(20.0);

            // Display cues with progress bars
            egui::ScrollArea::vertical().show(ui, |ui| {
                for cue in &self.cues {
                    ui.horizontal(|ui| {
                        let active_color = if cue.is_playing {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else {
                            egui::Color32::from_rgb(150, 150, 150)
                        };

                        ui.label(egui::RichText::new(&cue.name).color(active_color).strong());

                        ui.label(
                            egui::RichText::new(Self::format_duration(cue.start_time))
                                .color(active_color),
                        );

                        // Progress bar
                        let progress_height = 8.0;
                        let progress_response = ui.add(
                            egui::ProgressBar::new(cue.progress)
                                .desired_width(200.0)
                                .desired_height(progress_height),
                        );

                        // Show duration on hover
                        if progress_response.hovered() {
                            egui::show_tooltip(
                                ui.ctx(),
                                progress_response.layer_id,
                                egui::Id::new("duration_tooltip"),
                                |ui| {
                                    ui.label(format!("Duration: {}s", cue.duration.as_secs()));
                                },
                            );
                        }
                    });
                }
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
