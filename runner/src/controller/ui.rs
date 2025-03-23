use crate::user_event::UserEvent;
use egui_winit::winit::event_loop::EventLoopProxy;

impl super::Controller {
    pub fn ui(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        _event_proxy: &EventLoopProxy<UserEvent>,
    ) {
        ui.add(egui::Label::new(" Simulation Speed").selectable(false));
        ui.add(
            egui::Slider::new(&mut self.simulation_runner.speed, 0.01..=99.0)
                .logarithmic(true)
                .max_decimals(2),
        );
        ui.checkbox(&mut self.simulation_runner.paused, "Paused");
        ui.checkbox(&mut self.debug, "Debug");
        if self.debug {
            egui::Grid::new("debug_grid").show(ui, |ui| {
                ui.label("Elapsed");
                ui.label(format!("{:.1}s", self.start.elapsed().as_secs_f64()));
                ui.end_row();

                ui.label("Zoom");
                ui.label(format!("{:.1}x", self.camera.zoom));
                ui.end_row();

                ui.label("Translate");
                ui.label(format!("{:.2}", self.camera.translate));
                ui.end_row();
            });
        }
    }
}
