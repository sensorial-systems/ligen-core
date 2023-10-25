pub use crate::prelude::*;

use egui::ComboBox;
use ligen_ir::{Float, Integer};

#[derive(Default)]
pub struct Primitive {
    editable: bool
}

impl Primitive {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn editable(&mut self, editable: bool) -> &mut Self {
        self.editable = editable;
        self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, type_: &mut ligen_ir::Primitive) {
        if self.editable {
            let list = [
                ligen_ir::Primitive::Float(Float::F32),
                ligen_ir::Primitive::Float(Float::F64),
                ligen_ir::Primitive::Integer(Integer::I8),
                ligen_ir::Primitive::Integer(Integer::I16),
                ligen_ir::Primitive::Integer(Integer::I32),
                ligen_ir::Primitive::Integer(Integer::I64),
                ligen_ir::Primitive::Integer(Integer::I128),
                ligen_ir::Primitive::Integer(Integer::ISize),
                ligen_ir::Primitive::Integer(Integer::U8),
                ligen_ir::Primitive::Integer(Integer::U16),
                ligen_ir::Primitive::Integer(Integer::U32),
                ligen_ir::Primitive::Integer(Integer::U64),
                ligen_ir::Primitive::Integer(Integer::U128),
                ligen_ir::Primitive::Integer(Integer::USize),
                ligen_ir::Primitive::Boolean,
                ligen_ir::Primitive::Character
            ];
            ComboBox::new("primitive", "")
                .selected_text(type_.to_string())
                .show_ui(ui, |ui| {
                    for item in list {
                        ui.selectable_value(type_, item, item.to_string());
                    }
                });
        } else {
            ui.label(type_.to_string());
        }
    }
}