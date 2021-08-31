use ligen::generator::{ImplementationVisitor, FileProcessorVisitor, FileSet, FunctionVisitor, ParameterVisitor, FileGeneratorVisitors, StructureVisitor, ObjectVisitor, ModuleVisitor, ProjectVisitor};
use ligen::ir;
use std::path::PathBuf;
use crate::generator::CSharpGenerator;
use ligen::conventions::naming::PascalCase;

/// Project processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct ProjectProcessor;

/// Module processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct ModuleProcessor;

/// Object processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct ObjectProcessor;

/// Structure processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct StructureProcessor;

/// Implementation processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct ImplementationProcessor;

/// Function processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct FunctionProcessor;

/// Parameter processor.
#[derive(Default, Clone, Copy, Debug)]
pub struct ParameterProcessor;

fn path(object: &ObjectVisitor) -> PathBuf {
    let mut path = PathBuf::from("");
    for segment in &object.path.segments {
        path = path.join(segment.to_string());
    }
    path.with_extension("cs")
}

impl FileProcessorVisitor for ProjectProcessor {
    type Visitor = ProjectVisitor;

    fn process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FileProcessorVisitor for ModuleProcessor {
    type Visitor = ModuleVisitor;

    fn process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FileProcessorVisitor for ObjectProcessor {
    type Visitor = ObjectVisitor;

    fn process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FileProcessorVisitor for StructureProcessor {
    type Visitor = StructureVisitor;

    fn process(&self, file_set: &mut FileSet, structure: &Self::Visitor) {
        let file = file_set.entry(&path(&structure.parent));
        let name = structure.parent.parent.parent_project().name();
        let name = PascalCase::from(name.clone());
        file.writeln(format!("namespace {}", name));
        file.writeln("{");
        file.writeln("\tusing System.Runtime.InteropServices;");
        file.writeln("\t[StructLayout(LayoutKind.Sequential, Pack = 1)]");
        file.writeln(format!("\tpublic struct {}", structure.identifier));
        file.writeln("\t{");

        let fields: Vec<_> = structure
            .fields
            .iter()
            .map(|field| (field.type_.clone(), field.identifier.clone()))
            .collect();

        for (type_, identifier) in &fields {
            file.writeln(format!("\t\tpublic readonly {} {};", type_, identifier));
        }
        file.writeln("");

        let arguments = fields
            .iter()
            .map(|(type_, identifier)| format!("{} {}", type_, identifier))
            .collect::<Vec<_>>()
            .join(", ");

        file.writeln(format!("\t\tpublic {}({})", structure.identifier, arguments));
        file.writeln("\t\t{");

        for (_, identifier) in fields {
            file.writeln(format!("\t\t\tthis.{identifier} = {identifier};", identifier = identifier));
        }

        file.writeln("\t\t}");
        file.writeln("\t}");
        file.writeln("}");

    }

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FileProcessorVisitor for ImplementationProcessor {
    type Visitor = ImplementationVisitor;

    fn process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FunctionProcessor {
    /// Generate function name.
    pub fn generate_function_name(&self, _visitor: &FunctionVisitor) -> String {
        Default::default()
    }

    /// Generate function output.
    pub fn generate_function_output(&self, _output: &Option<ir::Type>) -> String {
        Default::default()
    }
}

impl FileProcessorVisitor for FunctionProcessor {
    type Visitor = FunctionVisitor;

    fn process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FileProcessorVisitor for ParameterProcessor {
    type Visitor = ParameterVisitor;

    fn process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}

    fn post_process(&self, _file_set: &mut FileSet, _visitor: &Self::Visitor) {}
}

impl FileGeneratorVisitors for CSharpGenerator {
    type ProjectProcessor = ProjectProcessor;
    type ModuleProcessor = ModuleProcessor;
    type ObjectProcessor = ObjectProcessor;
    type StructureProcessor = StructureProcessor;
    type ImplementationProcessor = ImplementationProcessor;
    type FunctionProcessor = FunctionProcessor;
    type ParameterProcessor = ParameterProcessor;
}
