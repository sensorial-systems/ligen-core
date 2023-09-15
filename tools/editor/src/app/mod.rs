use egui::{Style, Visuals};
use ligen_ir::Project;
pub mod ui;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    project: Option<Project>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let project = Project {
            directory: Default::default(),
            name: "Project".try_into().unwrap(),
            root_module: Default::default()
        };
        let project = Some(project);
        Self { project }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        cc.egui_ctx.set_style(Style {
            visuals: Visuals::dark(),
            ..Default::default()
        });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        if let Some(project) = self.project.as_ref() {
                            let directory = project
                                    .directory
                                    .display()
                                    .to_string();
                            let file = rfd::FileDialog::new()
                                .add_filter("ligen-ir", &["lir"])
                                .set_directory(directory)
                                .save_file();
                            if let Some(file) = file {
                                project.save(file).ok();
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Load").clicked() {
                        if let Some(project) = self.project.as_ref() {
                            let directory = project
                                .directory
                                .display()
                                .to_string();
                            let file = rfd::FileDialog::new()
                                .add_filter("ligen-ir", &["lir"])
                                .set_directory(directory)
                                .pick_file();
                            if let Some(file) = file {
                                if let Ok(project) = Project::load(file) {
                                    self.project = Some(project);
                                }
                            }
                            ui.close_menu();
                        }
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(project) = &mut self.project {
                ui::Project::new().show(ui, project);
            } else {
            }
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
