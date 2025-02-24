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

struct BeatIndicator {
    current_beat: usize,
    last_beat_time: Instant,
    beat_duration: Duration,
}

impl BeatIndicator {
    fn new() -> Self {
        Self {
            current_beat: 0,
            last_beat_time: Instant::now(),
            beat_duration: Duration::from_secs_f32(60.0 / 120.0), // Default 120 BPM
        }
    }

    fn update(&mut self, bpm: f32) {
        self.beat_duration = Duration::from_secs_f32(60.0 / bpm);
        if self.last_beat_time.elapsed() >= self.beat_duration {
            self.current_beat = (self.current_beat + 1) % 4;
            self.last_beat_time = Instant::now();
        }
    }
}

enum AppView {
    Timeline,
    Patch,
}

struct HaloApp {
    current_view: AppView,
    running: bool,
    start_time: Option<Instant>,
    elapsed: Duration,
    show_system_time: bool,
    cues: Vec<Cue>,
    link_enabled: bool,
    bpm: f32,
    fps: f32,
    effects_count: usize,
    pad_states: Vec<(bool, String)>, // (is_active, label)
    beat_indicator: BeatIndicator,
}

impl Default for HaloApp {
    fn default() -> Self {
        Self {
            current_view: AppView::Timeline,
            running: false,
            start_time: None,
            elapsed: Duration::from_secs(0),
            show_system_time: false,
            cues: vec![
                Cue::new("Opening", 2, 5),
                Cue::new("First Verse", 8, 10),
                Cue::new("Chorus", 19, 8),
                Cue::new("Bridge", 28, 12),
                Cue::new("Finale", 41, 6),
            ],
            link_enabled: false,
            bpm: 120.0,
            fps: 44.0,
            effects_count: 3,
            pad_states: vec![
                (false, "Smoke".to_string()),
                (false, "Strobe".to_string()),
                (false, "Laser".to_string()),
                (false, "Flash".to_string()),
                (false, "Burst".to_string()),
                (false, "Pulse".to_string()),
                (false, "Wave".to_string()),
                (false, "Spark".to_string()),
                (false, "Fade".to_string()),
                (false, "Chase".to_string()),
                (false, "Sweep".to_string()),
                (false, "Blast".to_string()),
            ],
            beat_indicator: BeatIndicator::new(),
        }
    }
}

impl HaloApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load custom fonts
        let mut fonts = egui::FontDefinitions::default();

        // Add the LED font
        fonts.font_data.insert(
            "matrix".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/digital-7-mono.ttf"
            ))),
        );

        fonts.families.insert(
            egui::FontFamily::Name("matrix".into()),
            vec!["matrix".into()],
        );

        _cc.egui_ctx.set_fonts(fonts);

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

    fn format_system_time(&self) -> String {
        let now = chrono::Local::now();
        now.format("%H:%M:%S.%3f").to_string()
    }

    fn draw_beat_indicator(&mut self, ui: &mut egui::Ui) {
        let size = 24.0;
        let spacing = 2.0;
        let inner_size = (size - spacing * 2.0) / 2.0;

        let (id, rect) = ui.allocate_space(egui::vec2(size, size));
        let painter = ui.painter();

        // Draw outer square
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::GREEN),
            egui::StrokeKind::Outside,
        );

        // Draw inner squares
        for i in 0..4 {
            let row = i / 2;
            let col = i % 2;
            let pos = rect.min
                + egui::vec2(
                    spacing + col as f32 * (inner_size + spacing),
                    spacing + row as f32 * (inner_size + spacing),
                );

            let inner_rect = egui::Rect::from_min_size(pos, egui::vec2(inner_size, inner_size));

            let color = if i == self.beat_indicator.current_beat {
                egui::Color32::GREEN
            } else {
                egui::Color32::DARK_GREEN
            };

            painter.rect_filled(inner_rect, 0.0, color);
        }

        if self.running {
            self.beat_indicator.update(self.bpm);
        }
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

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Halo", |ui| {
                    if ui.button("About").clicked() {
                        // Add about dialog logic here
                    }
                    if ui.button("Load Show").clicked() {
                        // Add about dialog logic here
                    }
                    if ui.button("Save Show").clicked() {
                        // Add about dialog logic here
                    }
                    if ui.button("Quit").clicked() {
                        // Add quit logic here
                    }
                });
            });
        });

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.draw_beat_indicator(ui);
                ui.add_space(8.0);

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

                // Right side elements
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Patch").clicked() {
                        self.current_view = match self.current_view {
                            AppView::Timeline => AppView::Patch,
                            AppView::Patch => AppView::Timeline,
                        };
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                AppView::Timeline => {
                    // Use large text for the timecode display
                    let font_id = egui::FontId::new(120.0, egui::FontFamily::Name("matrix".into()));

                    ui.spacing_mut().item_spacing.y = 20.0;

                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        // Add toggle button here
                        if ui
                            .button(if self.show_system_time {
                                "Show Timecode"
                            } else {
                                "Show System Time"
                            })
                            .clicked()
                        {
                            self.show_system_time = !self.show_system_time;
                        }

                        ui.label(
                            egui::RichText::new(if self.show_system_time {
                                self.format_system_time()
                            } else {
                                self.format_timecode()
                            })
                            .font(font_id)
                            .color(egui::Color32::GREEN),
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

                                ui.label(
                                    egui::RichText::new(&cue.name).color(active_color).strong(),
                                );

                                ui.label(
                                    egui::RichText::new(Self::format_duration(cue.start_time))
                                        .color(active_color),
                                );

                                // Progress bar
                                let progress_response = ui.add(
                                    egui::ProgressBar::new(cue.progress)
                                        .desired_width(200.0)
                                        .desired_height(30.0)
                                        .corner_radius(0.0),
                                );

                                // Show duration on hover
                                if progress_response.hovered() {
                                    egui::show_tooltip(
                                        ui.ctx(),
                                        progress_response.layer_id,
                                        egui::Id::new("duration_tooltip"),
                                        |ui| {
                                            ui.label(format!(
                                                "Duration: {}s",
                                                cue.duration.as_secs()
                                            ));
                                        },
                                    );
                                }
                            });
                        }
                    });

                    ui.add_space(20.0);
                    ui.label("Override Pads");
                    ui.add_space(10.0);

                    egui::Grid::new("midi_pads")
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            for (i, (active, label)) in self.pad_states.iter_mut().enumerate() {
                                let response = ui.add(
                                    egui::Button::new(egui::RichText::new(format!("{}", label)))
                                        .min_size(egui::vec2(80.0, 80.0))
                                        .fill(if *active {
                                            egui::Color32::from_rgb(100, 200, 100)
                                        } else {
                                            egui::Color32::from_rgb(60, 60, 60)
                                        }),
                                );

                                if response.clicked() {
                                    *active = !*active;
                                    // Here you can add MIDI handling logic
                                }

                                if (i + 1) % 4 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                }
                AppView::Patch => {
                    // Your new patch view code here
                    ui.heading("Patch Editor");
                    // Add patch editor UI elements
                }
            }
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.1}", self.fps));
                ui.separator();
                ui.label(format!("Active Effects: {}", self.effects_count));
            });
        });

        // Request continuous repaint while running
        if self.running || self.show_system_time {
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
            maximized: Some(true),
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
