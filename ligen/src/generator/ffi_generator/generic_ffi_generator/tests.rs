use std::convert::TryFrom;
use crate::prelude::*;
use crate::ir::{Project, Module, Visibility, Function, Type, Float, Object, Structure, Implementation, ImplementationItem};
use crate::generator::{GenericFFIGenerator, File, ProjectVisitor, ModuleVisitor, FunctionVisitor};
use crate::marshalling::Marshaller;
use crate::conventions::naming::{NamingConvention, KebabCase};
use std::path::PathBuf;

pub struct Generator;

impl GenericFFIGenerator for Generator {}

fn mock_module(root_module: Module) -> ProjectVisitor {
    let mut project = Project {
        name: NamingConvention::KebabCase(KebabCase::try_from("project-mock").unwrap()),
        root_module,
        path: PathBuf::from("project-mock-path"),
        manifest_path: PathBuf::from("project-mock-path/Cargo.toml")
    };
    project.root_module.replace_self_with_explicit_names();
    project.into()
}

fn mock_function(function: TokenStream) -> FunctionVisitor {
    let function = Function::from(function);
    let module = Module {
        attributes: Default::default(),
        name: "module_mock".into(),
        visibility: Visibility::Public,
        imports: Default::default(),
        objects: Default::default(),
        modules: Default::default(),
        functions: vec![function]
    };
    let project_visitor = mock_module(module);
    let module_visitor = ModuleVisitor::from(&project_visitor.child(project_visitor.root_module.clone()));
    let function_visitor = FunctionVisitor::from(&module_visitor.child(module_visitor.functions[0].clone()));
    function_visitor
}

fn mock_method(method: TokenStream) -> Result<FunctionVisitor> {
    let function = Function::from(method);
    let module = Module {
        attributes: Default::default(),
        name: "module_mock".into(),
        visibility: Visibility::Public,
        imports: Default::default(),
        modules: Default::default(),
        functions: Default::default(),
        objects: vec! [
            Object {
                path: "ObjectMock".into(),
                definition: Structure {
                    attributes: Default::default(),
                    visibility: Visibility::Public,
                    identifier: "ObjectMock".into(),
                    fields: Default::default()
                }.into(),
                implementations: vec![
                    Implementation {
                        attributes: Default::default(),
                        self_: Type::Compound("ObjectMock".into()),
                        items: vec![
                            ImplementationItem::Method(function)
                        ]
                    }
                ]
            }
        ]
    };
    let project_visitor = mock_module(module);
    let module_visitor = ModuleVisitor::from(&project_visitor.child(project_visitor.root_module.clone()));
    let object_visitor = module_visitor.child(module_visitor.current.objects[0].clone());
    let implementation_visitor = object_visitor.child(object_visitor.current.implementations[0].clone());
    if let ImplementationItem::Method(function) = implementation_visitor.current.items[0].clone() {
        let function_visitor = FunctionVisitor::from(&implementation_visitor.child(function));
        Ok(function_visitor)
    } else {
        Err(Error::Message("Failed to get function visitor.".into()))
    }
}

#[test]
fn test_static_method() -> Result<()> {
    let marshaller = Marshaller::new();
    let function_visitor = mock_method(quote! {
        pub fn add(a: f32, b: f32) -> f32 {
            a + b
        }
    })?;
    let mut file = File::new(PathBuf::from(""), Default::default());
    Generator::generate_function(&marshaller, &mut file, function_visitor);
    let mut ffi = String::new();
    ffi.push_str("#[no_mangle]\n");
    ffi.push_str("pub extern fn ObjectMock_add(a: f32, b: f32, ) -> f32 {\n");
    ffi.push_str("\tlet result = project_mock::ObjectMock::add(a.into(), b.into(), );\n");
    ffi.push_str("\tresult.into()\n");
    ffi.push_str("}\n");
    assert_eq!(file.content, ffi);
    Ok(())
}

#[test]
fn test_method() -> Result<()> {
    let marshaller = Marshaller::new();
    let function_visitor = mock_method(quote! {
        pub fn add(&self, b: f32) -> f32 {
            self.a + b
        }
    })?;
    let mut file = File::new(PathBuf::from(""), Default::default());
    Generator::generate_function(&marshaller, &mut file, function_visitor);
    let mut ffi = String::new();
    ffi.push_str("#[no_mangle]\n");
    ffi.push_str("pub extern fn ObjectMock_add(self: &ObjectMock, b: f32, ) -> f32 {\n");
    ffi.push_str("\tlet result = project_mock::ObjectMock::add(self.into(), b.into(), );\n");
    ffi.push_str("\tresult.into()\n");
    ffi.push_str("}\n");
    assert_eq!(file.content, ffi);
    Ok(())
}

#[test]
fn test_function() -> Result<()> {
    let marshaller = Marshaller::new();
    let function_visitor = mock_function(quote! {
        pub fn add(a: f32, b: f32) -> f32 {
            a + b
        }
    });
    let mut file = File::new(PathBuf::from(""), Default::default());
    Generator::generate_function(&marshaller, &mut file, function_visitor);
    let mut ffi = String::new();
    ffi.push_str("#[no_mangle]\n");
    ffi.push_str("pub extern fn add(a: f32, b: f32, ) -> f32 {\n");
    ffi.push_str("\tlet result = project_mock::add(a.into(), b.into(), );\n");
    ffi.push_str("\tresult.into()\n");
    ffi.push_str("}\n");
    assert_eq!(file.content, ffi);
    Ok(())
}

#[test]
fn test_marshalled_function() -> Result<()> {
    let mut marshaller = Marshaller::new();
    marshaller.add_input_marshalling(Type::Compound("Instant".into()), Float::F64.into());
    marshaller.add_output_marshalling(Type::Compound("Duration".into()), Float::F32.into());
    let function_visitor = mock_function(quote! {
        pub fn subtract(a: Instant, b: Instant) -> Duration {
            a + b
        }
    });
    let mut file = File::new(PathBuf::from(""), Default::default());
    Generator::generate_function(&marshaller, &mut file, function_visitor);
    let mut ffi = String::new();
    ffi.push_str("#[no_mangle]\n");
    ffi.push_str("pub extern fn subtract(a: f64, b: f64, ) -> f32 {\n");
    ffi.push_str("\tlet result = project_mock::subtract(a.into(), b.into(), );\n");
    ffi.push_str("\tresult.into()\n");
    ffi.push_str("}\n");
    assert_eq!(file.content, ffi);
    Ok(())
}
