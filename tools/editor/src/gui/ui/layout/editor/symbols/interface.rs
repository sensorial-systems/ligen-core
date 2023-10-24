use crate::gui::ui::EditableList;
use crate::gui::ui::editor::ir::Identifier;
use crate::prelude::*;

pub struct Interface;

impl Interface {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, interface: &mut ligen_ir::Interface) {
        ui.vertical(|ui| {
            Identifier::new().show(ui, &mut interface.identifier);
            EditableList::new("Objects", "Add object").show(ui, &mut interface.objects, |ui, object| {
                Identifier::new().show(ui, &mut object.identifier);
            });
            EditableList::new("Functions", "Add function").show(ui, &mut interface.functions, |ui, function| {
                Identifier::new().show(ui, &mut function.identifier);
            });
            EditableList::new("Methods", "Add method").show(ui, &mut interface.methods, |ui, method| {
                Identifier::new().show(ui, &mut method.identifier);
            });
        });
    }
}