use eframe::{egui, epi};
use log::{info, warn};
use porsmo::pomodoro::*;
use porsmo_helpers::{alert_pomo, fmt_time};
use std::time::Duration;

// #[derive(Default)]
struct Porsmo {
    pomo: Pomodoro,
    alerted: bool,
}

impl Default for Porsmo {
    fn default() -> Self {
        Self {
            pomo: Pomodoro::new(
                Duration::from_secs(25 * 60),
                Duration::from_secs(5 * 60),
                Duration::from_secs(10 * 60),
                alert_pomo,
            ),
            alerted: false,
        }
    }
}

impl Porsmo {
    fn configure_font(&mut self, _ctx: &egui::Context) {
        let _font_def = egui::FontDefinitions::default();
        warn!("No custom font actually implemented");
    }
}

impl Porsmo {
    fn alert(&mut self) {
        self.alerted = true;
        alert_pomo(self.pomo.check_next_mode());
    }

    fn has_alerted(&self) -> bool {
        self.alerted
    }

    fn reset(&mut self) {
        self.alerted = false;
    }

    fn reset_timer(&mut self) {
        self.pomo.reset();
        self.reset();
    }
}

impl epi::App for Porsmo {
    fn name(&self) -> &str {
        "Porsmo"
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.configure_font(ctx);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.name());

            if self.pomo.has_ended() {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }

            match self.pomo.checked_counter_at() {
                CountType::Count(c) => {
                    if self.pomo.is_paused() {
                        ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                    }

                    ui.heading(format!("{}", fmt_time(c.as_secs())));
                    ui.visuals_mut().override_text_color = None;
                }
                CountType::Exceed(c) => {
                    if !self.has_alerted() {
                        self.alert();
                    }

                    if self.pomo.is_paused() {
                        ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                    }
                    ui.heading(format!("+{}", fmt_time(c.as_secs())));
                    ui.visuals_mut().override_text_color = None;
                }
            };

            ui.horizontal(|ui| {
                if ui
                    .button(if self.pomo.is_running() {
                        "Pause"
                    } else {
                        "Resume"
                    })
                    .clicked()
                {
                    self.pomo.toggle();
                }

                if ui.button("Reset").clicked() {
                    self.reset_timer();
                }

                if ui.button("Start next").clicked() {
                    self.reset();
                    self.pomo.next_mode();
                }
            });

            if ui.input().key_pressed(egui::Key::Space) {
                self.pomo.toggle();
            } else if ui.input().key_pressed(egui::Key::Enter) {
                if self.pomo.has_ended() {
                    self.pomo.next_mode();
                } else {
                    self.pomo.toggle();
                }
            }
        });

        ctx.request_repaint();
        frame.set_window_size(ctx.used_size());
    }
}

fn main() {
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

    let app = Porsmo::default();
    let mut native_options = eframe::NativeOptions::default();
    let win_size = egui::Vec2::new(225., 100.);

    native_options.initial_window_size = Some(win_size);
    info!("window size: {:?}", win_size);

    eframe::run_native(Box::new(app), native_options);
}
