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
            egui::Slider::new(&mut self.speed, 0.01..=99.0)
                .logarithmic(true)
                .max_decimals(2),
        );
        ui.checkbox(&mut self.paused, "Paused");
        ui.add(egui::Label::new("           Zoom").selectable(false));
        ui.add(
            egui::Slider::new(&mut self.zoom, 1.0..=100.0)
                .logarithmic(true)
                .max_decimals(2),
        );
        ui.checkbox(&mut self.debug, "Debug");
        if self.debug {
            ui.label(format!(
                "Elapsed: {:.1}s",
                self.start.elapsed().as_secs_f64()
            ));
        }
    }
}
