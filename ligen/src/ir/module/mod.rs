//! Module representation.

mod import;
pub use import::*;

use crate::prelude::*;
use crate::ir::{Object, Path, Structure, Implementation, Visibility, Identifier, TypeDefinition, Enumeration, Attributes, Attribute, Function};
use std::convert::TryFrom;
use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use proc_macro2::TokenStream;
use syn::parse_quote::parse;

/// Module representation.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// Attributes.
    pub attributes: Attributes,
    /// Visibility.
    pub visibility: Visibility,
    /// Module name.
    pub name: Identifier,
    /// Imports.
    pub imports: Vec<Import>,
    /// Sub-modules.
    pub modules: Vec<Module>,
    /// Functions.
    pub functions: Vec<Function>,
    /// Objects.
    pub objects: Vec<Object>
}

impl Module {
    /// Tells if ligen is ignoring this module.
    pub fn ignored(&self) -> bool {
        Self::ignored_from_attributes(&self.attributes)
    }

    /// Find the Type definition.
    pub fn find_definition(&self, path: &Path) -> Option<TypeDefinition> {
        if let Some(identifier) = path.segments.first() {
            if *identifier == self.name {
                let mut path = path.clone();
                path.segments.remove(0);
                if path.segments.len() > 1 {
                    self
                        .modules
                        .iter()
                        .filter_map(|module| module.find_definition(&path))
                        .next()
                } else {
                    if let Some(identifier) = path.segments.first() {
                        self
                            .objects
                            .iter()
                            .filter(|object| object.definition.identifier() == identifier)
                            .map(|object| object.definition.clone())
                            .next()
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn ignored_from_attributes(attributes: &Attributes) -> bool {
        attributes.contains(&Attribute::Group("ligen".into(), Attribute::Group("ignore".into(), Default::default()).into()))
    }
}

impl TryFrom<&std::path::Path> for Module {
    type Error = Error;
    fn try_from(from: &std::path::Path) -> Result<Self> {
        // FIXME: This function needs a cleanup.
        let mut file = File::open(from)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;
        let file = syn::parse_file(&src)?;
        let visibility = Visibility::Public;
        let parent_path = from.parent().expect("Failed to get parent path.");

        // FIXME: This is repetitive.
        let mut name = from
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .filter(|name| name != "mod");
        if name.is_none() {
            name = parent_path
                .file_stem()
                .and_then(|parent| parent.to_str())
                .map(|parent| parent.to_string());
        }
        let name = name
            .ok_or_else(|| Error::Message("Couldn't get module name.".into()))?
            .into();

        // FIXME: This needs a better generalization.
        // println!("Parent: {}", parent_path.display());
        let base_path = if from.ends_with("src/lib.rs") || from.ends_with("mod.rs") {
            from.parent().expect("Couldn't get parent.").to_path_buf()
        } else {
            from.with_extension("")
        };
        let attributes = Self::parse_ligen_attributes(&file.attrs, &file.items)?;
        let ignored = Self::ignored_from_attributes(&attributes);
        let (modules, objects) = Self::extract_modules_and_objects(ignored, &file.items, base_path.as_path())?;
        let imports = Imports::try_from(file.items.as_slice())?.0;
        let functions = Self::extract_functions(file.items.as_slice());
        Ok(Module { attributes, imports, visibility, name, modules, functions, objects })
    }
}

impl Module {
    /// Replace all the occurrences of `Self` by the real object name.
    /// e.g.:
    /// ```rust,compile_fail
    /// impl Object {
    ///     fn f(self: &Self) {}
    /// }
    /// ```
    /// becomes
    /// ```rust,compile_fail
    /// impl Object {
    ///     fn f(self: &Object) {}
    /// }
    /// ```
    pub fn replace_self_with_explicit_names(&mut self) {
        for module in &mut self.modules {
            module.replace_self_with_explicit_names();
        }
        for object in &mut self.objects {
            for implementation in &mut object.implementations {
                implementation.replace_self_with_explicit_names();
            }
        }
    }

    /// Find the module with the specified path.
    pub fn find_module(&self, path: &Path) -> Option<&Module> {
        let mut path = path.clone();
        if let Some(identifier) = path.pop_front() {
            let module = self
                .modules
                .iter()
                .find(|module| identifier == module.name);
            if let Some(module) = module {
                if path.segments.is_empty() {
                    Some(module)
                } else {
                    module.find_module(&path)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Replace wild card imports with actual imports.
    pub fn replace_wildcard_imports(&mut self) {
        for module in &mut self.modules {
            module.replace_wildcard_imports();
        }
        let wildcard_imports: Vec<_> = self
            .imports
            .iter()
            .filter(|import| import.path.last() == "*".into())
            .cloned()
            .collect();
        let mut imports: Vec<_> = self
            .imports
            .iter()
            .filter(|import| import.path.last() != "*".into())
            .cloned()
            .collect();
        for import in wildcard_imports {
            let module_path = import.path.clone().without_last();
            if let Some(module) = self.find_module(&module_path) {
                for object in &module.objects {
                    if let Visibility::Public = object.definition.visibility() {
                        imports.push(Import {
                            attributes: import.attributes.clone(),
                            visibility: import.visibility.clone(),
                            renaming: import.renaming.clone(),
                            path: module_path.clone().join(object.definition.identifier().clone())
                        })
                    }
                }
                for internal_import in &module.imports {
                    if let Visibility::Public = internal_import.visibility {
                        let identifier = if let Some(renaming) = &internal_import.renaming {
                            renaming.clone()
                        } else {
                            internal_import.path.last()
                        };
                        imports.push(Import {
                            attributes: import.attributes.clone(),
                            visibility: import.visibility.clone(),
                            renaming: import.renaming.clone(),
                            path: module_path.clone().join(identifier)
                        })
                    }
                }
            }
        }
        self.imports = imports;
    }


}

impl Module {
    fn extract_functions(items: &[syn::Item]) -> Vec<Function> {
        let mut functions = Vec::new();
        for item in items {
            if let syn::Item::Fn(function) = item {
                functions.push(function.clone().into());
            }
        }
        functions
    }

    // FIXME: Find a better place for this function.
    fn extract_modules_and_objects(ignored: bool, items: &[syn::Item], base_path: &std::path::Path) -> Result<(Vec<Module>, Vec<Object>)> {
        if ignored {
            Ok((Default::default(), Default::default()))
        } else {
            let modules = Module::parse_modules(&items, base_path)?;
            let objects = Module::parse_objects(&items)?;
            Ok((modules, objects))
        }
    }

    fn parse_modules(items: &[syn::Item], base_path: &std::path::Path) -> Result<Vec<Module>> {
        let mut modules = Vec::new();
        for item in items {
            match item {
                syn::Item::Mod(module) => {
                    let module = Module::try_from((module.clone(), base_path))?;
                    if !module.ignored() {
                        modules.push(module)
                    }
                },
                _ => ()
            }
        }
        Ok(modules)
    }

    fn parse_ligen_attributes(attrs: &Vec<syn::Attribute>, items: &[syn::Item]) -> Result<Attributes> {
        let mut attributes: Attributes = attrs.clone().try_into()?;
        for item in items {
            match item {
                syn::Item::Macro(call) => {
                    let attribute = Attribute::try_from(call.clone())?;
                    if let Attribute::Group(identifier, grouped_attributes) = &attribute {
                        if *identifier == Identifier::from("inner_ligen") {
                            attributes.attributes.push(Attribute::Group("ligen".into(), grouped_attributes.clone()));
                        }
                    }
                },
                _ => ()
            }
        }
        Ok(attributes)
    }

    fn parse_objects(items: &[syn::Item]) -> Result<Vec<Object>> {
        let mut objects: HashMap<Path, (Option<TypeDefinition>, Vec<Implementation>)> = HashMap::new();
        for item in items {
            match item {
                syn::Item::Enum(enumeration) => {
                    let enumeration = Enumeration::try_from(enumeration.clone())?;
                    let path = enumeration.identifier.clone().into();
                    let definition = Some(TypeDefinition::Enumeration(enumeration));
                    if let Some((optional_definition, _)) = objects.get_mut(&path) {
                        *optional_definition = definition;
                    } else {
                        objects.insert(path, (definition, Default::default()));
                    }
                },
                syn::Item::Struct(structure) => {
                    let structure = Structure::try_from(structure.clone())?;
                    let path = structure.identifier.clone().into();
                    let definition = Some(TypeDefinition::Structure(structure));
                    if let Some((optional_definition, _implementations)) = objects.get_mut(&path) {
                        *optional_definition = definition;
                    } else {
                        objects.insert(path, (definition, Default::default()));
                    }
                },
                syn::Item::Impl(implementation) => {
                    // TODO: Consider `impl Trait for Object`?
                    if implementation.trait_.is_none() {
                        let implementation = Implementation::try_from(implementation.clone())?;
                        let path = implementation.self_.path();
                        if let Some((_definition, implementations)) = objects.get_mut(&path) {
                            implementations.push(implementation);
                        } else {
                            objects.insert(path, (None, vec![implementation]));
                        }
                    }
                }
                _ => ()
            }
        }
        let mut objects: Vec<_> = objects
            .into_iter()
            .map(|(path, (definition, implementations))| Object {
                // FIXME: This shouldn't use expect
                definition: definition.expect(&format!("Type definition for {} not found.", path)),
                path,
                implementations
            })
            .collect();
        objects.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(objects)
    }
}

#[allow(unused_qualifications)]
impl TryFrom<TokenStream> for Module {
    type Error = Error;
    fn try_from(tokenstream: TokenStream) -> Result<Self> {
        (parse::<syn::ItemMod>(tokenstream), std::path::Path::new("")).try_into()
    }
}

impl TryFrom<(syn::ItemMod, &std::path::Path)> for Module {
    type Error = Error;
    fn try_from(module: (syn::ItemMod, &std::path::Path)) -> Result<Self> {
        let (module, base_path) = module;
        let base_path = base_path.join(module.ident.to_string());
        if let Some((_, items)) = module.content {
            let attributes = Module::parse_ligen_attributes(&module.attrs, &items)?;
            let ignored = Module::ignored_from_attributes(&attributes);
            let (modules, objects) = Self::extract_modules_and_objects(ignored, &items, base_path.as_path())?;
            let name = module.ident.into();
            let visibility = module.vis.into();
            let imports = Imports::try_from(items.as_slice())?.0;
            let functions = Self::extract_functions(items.as_slice());
            Ok(Self { attributes, visibility, name, imports, modules, functions, objects })
        } else {
            let mut path = base_path.with_extension("rs");
            if !path.exists() {
                path = base_path.join("mod.rs");
            }
            let path = path.as_path();
            path.try_into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Object, Atomic, Integer, Type, Visibility, Function, Structure, Parameter, Implementation, ImplementationItem, Field, Attribute};
    use quote::quote;
    use pretty_assertions::assert_eq;

    #[test]
    fn imports() -> Result<()> {
        let module = quote! {
            mod root {
                mod object {
                    pub struct Object1;
                }
                mod objects {
                    pub struct Object2;
                    pub struct Object3;
                    struct Object4;
                    mod deeper {
                        pub struct Object5;
                        pub struct Object6;
                        struct Object7;
                    }
                    mod deeper2 {
                        pub struct Object8;
                        pub struct Object9;
                        pub struct ObjectA;
                    }
                    pub use deeper::*;
                    pub use deeper2::Object8;
                    use deeper2::Object9;
                    pub use deeper2::ObjectA as ObjectTen;
                }
                pub use object::Object1;
                pub use objects::*;
            }
        };
        let expected_module = quote! {
            mod root {
                mod object {
                    pub struct Object1;
                }
                mod objects {
                    pub struct Object2;
                    pub struct Object3;
                    struct Object4;
                    mod deeper {
                        pub struct Object5;
                        pub struct Object6;
                        struct Object7;
                    }
                    mod deeper2 {
                        pub struct Object8;
                        pub struct Object9;
                        pub struct ObjectA;
                    }
                    pub use deeper2::Object8;
                    use deeper2::Object9;
                    pub use deeper2::ObjectA as ObjectTen;
                    pub use deeper::Object5;
                    pub use deeper::Object6;
                }
                pub use object::Object1;
                pub use objects::Object2;
                pub use objects::Object3;
                pub use objects::Object8;
                pub use objects::ObjectTen;
                pub use objects::Object5;
                pub use objects::Object6;
            }
        };

        let expected_module = Module::try_from(expected_module)?;
        let mut module = Module::try_from(module)?;
        module.replace_wildcard_imports();
        assert_eq!(module, expected_module);
        Ok(())
    }

    #[test]
    fn object() -> Result<()> {
        let module = quote! {
            #[ligen(attribute)]
            mod objects {
                inner_ligen!(another_attribute);

                pub struct Object {
                    pub integer: i32
                }

                impl Object {
                    pub fn new(integer: i32) -> Self {
                        Self { integer }
                    }
                }

                pub struct AnotherObject;
            }
        };
        let module = Module::try_from(module)?;
        assert_eq!(
            module,
            Module {
                attributes: vec![
                    Attribute::Group("ligen".into(), Attribute::Group("attribute".into(), Default::default()).into()),
                    Attribute::Group("ligen".into(), Attribute::Group("another_attribute".into(), Default::default()).into()),
                ].into(),
                visibility: Visibility::Inherited,
                name: "objects".into(),
                imports: Default::default(),
                modules: Default::default(),
                functions: Default::default(),
                objects: vec![
                    Object {
                        path: "AnotherObject".into(),
                        definition: TypeDefinition::Structure(Structure {
                            attributes: Default::default(),
                            visibility: Visibility::Public,
                            identifier: "AnotherObject".into(),
                            fields: Default::default(),
                        }),
                        implementations: Default::default()
                    },
                    Object {
                        path: "Object".into(),
                        definition: TypeDefinition::Structure(Structure {
                            attributes: Default::default(),
                            visibility: Visibility::Public,
                            identifier: "Object".into(),
                            fields: vec![
                                Field {
                                    attributes: Default::default(),
                                    visibility: Visibility::Public,
                                    identifier: Some("integer".into()),
                                    type_: Type::Atomic(Atomic::Integer(Integer::I32))
                                }
                            ]
                        }),
                        implementations: vec![
                            Implementation {
                                attributes: Default::default(),
                                self_: Type::Compound("Object".into()),
                                items: vec![
                                    ImplementationItem::Method(Function {
                                        attributes: Default::default(),
                                        visibility: Visibility::Public,
                                        asyncness: None,
                                        identifier: "new".into(),
                                        inputs: vec![
                                            Parameter {
                                                attributes: Default::default(),
                                                identifier: "integer".into(),
                                                type_: Type::Atomic(Atomic::Integer(Integer::I32))
                                            }
                                        ],
                                        output: Some(Type::Compound("Self".into()))
                                    }
                                    )
                                ]
                            }
                        ]
                    }
                ]
            }
        );
        Ok(())
    }
}
