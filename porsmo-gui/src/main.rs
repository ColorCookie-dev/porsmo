use eframe::{egui, epi};
use log::{info, warn};
use porsmo::pomodoro::{CountType, Pomodoro};
use porsmo_helpers::fmt_time;

// #[derive(Default)]
struct Porsmo {
    pomo: Pomodoro,
}

impl Default for Porsmo {
    fn default() -> Self {
        Self {
            pomo: Pomodoro::default(),
        }
    }
}

impl Porsmo {
    fn configure_font(&mut self, _ctx: &egui::Context) {
        let _font_def = egui::FontDefinitions::default();
        warn!("No custom font actually implemented");
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
            match self.pomo.counter_at() {
                CountType::Count(c) => ui.heading(format!("{}", fmt_time(c.as_secs()))),
                CountType::Exceed(c) => ui.heading(format!("{}", fmt_time(c.as_secs()))),
            };

            ui.horizontal(|ui| {
                if ui.button("Pause/Resume").clicked() {
                    self.pomo.toggle();
                }
            });
        });

        ctx.request_repaint();
        frame.set_window_size(ctx.used_size());
    }
}

fn main() {
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

    let app = Porsmo::default();
    let mut native_options = eframe::NativeOptions::default();
    let win_size = egui::Vec2::new(250., 300.);

    native_options.initial_window_size = Some(win_size);
    info!("window size: {:?}", win_size);

    eframe::run_native(Box::new(app), native_options);
}
