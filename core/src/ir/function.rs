use crate::ir::{Attributes, Identifier, Parameter, Type};
use std::convert::{TryFrom, TryInto};
use syn::{ImplItemMethod, ItemFn};

#[derive(Debug, PartialEq, Copy, Clone)]
/// Async Struct
pub struct Async;

#[derive(Debug, PartialEq, Clone)]
/// Function Struct
pub struct Function {
    /// Attributes field.
    pub attributes: Attributes,
    /// Asyncness field.
    pub asyncness: Option<Async>,
    /// Identifier field.
    pub identifier: Identifier,
    /// Inputs field.
    pub inputs: Vec<Parameter>,
    /// Output field.
    pub output: Option<Type>,
}

macro_rules! impl_function {
    ($T:ident) => {
        impl From<$T> for Function {
            fn from(item_fn: $T) -> Self {
                let syn::Signature {
                    asyncness,
                    ident,
                    inputs,
                    output,
                    ..
                } = item_fn.sig;
                let inputs: Vec<Parameter> = inputs
                    .clone()
                    .into_iter()
                    .map(|x| x.try_into().expect("Failed to convert Parameter"))
                    .collect();
                let output: Option<Type> = match output {
                    syn::ReturnType::Default => None,
                    syn::ReturnType::Type(_x, y) => {
                        Some(Type::try_from(*y).expect("Failed to convert from ReturnType::Type"))
                    }
                };
                Self {
                    attributes: Attributes {
                        attributes: item_fn
                            .attrs
                            .into_iter()
                            .map(|x| x.parse_meta().expect("Failed to parse Meta").into())
                            .collect(),
                    },
                    asyncness: match asyncness {
                        Some(_x) => Some(Async),
                        None => None,
                    },
                    identifier: ident.into(),
                    inputs,
                    output,
                }
            }
        }
    };
}

impl_function!(ItemFn);
impl_function!(ImplItemMethod);

#[cfg(test)]
mod test {
    use super::{Async, Function, ImplItemMethod, ItemFn, Type};
    use crate::ir::{Attribute, Attributes, Identifier, Literal, Parameter, Reference, ReferenceKind};
    use quote::quote;
    use syn::parse_quote::parse;

    #[test]
    fn function() {
        assert_eq!(
            Function::from(parse::<ItemFn>(quote! {fn test() {}})),
            Function {
                attributes: Attributes { attributes: vec![] },
                asyncness: None,
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![],
                output: None
            }
        );
    }

    #[test]
    fn function_impl() {
        assert_eq!(
            Function::from(parse::<ImplItemMethod>(quote! {fn test() {}})),
            Function {
                attributes: Attributes { attributes: vec![] },
                asyncness: None,
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![],
                output: None
            }
        );
    }

    #[test]
    fn function_input() {
        assert_eq!(
            Function::from(parse::<ItemFn>(quote! {fn test(a: String, b: String) {}})),
            Function {
                attributes: Attributes { attributes: vec![] },
                asyncness: None,
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![
                    Parameter {
                        identifier: Identifier {
                            name: String::from("a")
                        },
                        type_: Type::Compound(Identifier {
                            name: String::from("String")
                        })
                    },
                    Parameter {
                        identifier: Identifier {
                            name: String::from("b")
                        },
                        type_: Type::Compound(Identifier {
                            name: String::from("String")
                        })
                    },
                ],
                output: None
            }
        );
    }

    #[test]
    fn function_output() {
        assert_eq!(
            Function::from(parse::<ItemFn>(quote! {fn test() -> String {}})),
            Function {
                attributes: Attributes { attributes: vec![] },
                asyncness: None,
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![],
                output: Some(Type::Compound(Identifier {
                    name: String::from("String")
                }))
            }
        );
    }

    #[test]
    fn function_input_output() {
        assert_eq!(
            Function::from(parse::<ItemFn>(
                quote! {fn test(a: String, b: &String, c: &mut String) -> &String {}}
            )),
            Function {
                attributes: Attributes { attributes: vec![] },
                asyncness: None,
                identifier: Identifier::new("test"),
                inputs: vec![
                    Parameter {
                        identifier: Identifier::new("a"),
                        type_: Type::Compound(Identifier::new("String"))
                    },
                    Parameter {
                        identifier: Identifier::new("b"),
                        type_: Type::Reference(
                            Reference {
                                kind: ReferenceKind::Borrow,
                                is_constant: true,
                                type_: Box::new(Type::Compound(Identifier::new("String")))
                            }
                        )
                    },
                    Parameter {
                        identifier: Identifier {
                            name: String::from("c")
                        },
                        type_: Type::Reference(
                            Reference {
                                kind: ReferenceKind::Borrow,
                                is_constant: false,
                                type_: Box::new(Type::Compound(Identifier::new("String")))
                            }
                        )
                    },
                ],
                output: Some(Type::Reference(
                    Reference {
                        kind: ReferenceKind::Borrow,
                        is_constant: true,
                        type_: Box::new(Type::Compound(Identifier::new("String")))
                    }
                ))
            }
        );
    }

    #[test]
    fn function_attribute() {
        assert_eq!(
            Function::from(parse::<ItemFn>(quote! {
                #[test(a = "b")]
                fn test() {}
            })),
            Function {
                attributes: Attributes {
                    attributes: vec![Attribute::Group(
                        Identifier::new("test"),
                        Attributes {
                            attributes: vec![Attribute::Named(
                                Identifier::new("a"),
                                Literal::String(String::from("b"))
                            )]
                        }
                    )]
                },
                asyncness: None,
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![],
                output: None
            }
        );
    }

    #[test]
    fn function_async() {
        assert_eq!(
            Function::from(parse::<ItemFn>(quote! {async fn test() {}})),
            Function {
                attributes: Attributes { attributes: vec![] },
                asyncness: Some(Async),
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![],
                output: None
            }
        );
    }

    #[test]
    fn function_complete() {
        assert_eq!(
            Function::from(parse::<ItemFn>(quote! {
            #[test(a = "b")]
                async fn test(a: String, b: &String, c: &mut String) -> &String {}
            })),
            Function {
                attributes: Attributes {
                    attributes: vec![Attribute::Group(
                        Identifier::new("test"),
                        Attributes {
                            attributes: vec![Attribute::Named(
                                Identifier::new("a"),
                                Literal::String(String::from("b"))
                            )]
                        }
                    )]
                },
                asyncness: Some(Async),
                identifier: Identifier {
                    name: String::from("test")
                },
                inputs: vec![
                    Parameter {
                        identifier: Identifier {
                            name: String::from("a")
                        },
                        type_: Type::Compound(Identifier {
                            name: String::from("String")
                        })
                    },
                    Parameter {
                        identifier: Identifier {
                            name: String::from("b")
                        },
                        type_: Type::Reference(
                            Reference {
                                kind: ReferenceKind::Borrow,
                                is_constant: true,
                                type_: Box::new(Type::Compound(Identifier::new("String")))
                            }
                        )
                    },
                    Parameter {
                        identifier: Identifier {
                            name: String::from("c")
                        },
                        type_: Type::Reference(
                            Reference {
                                kind: ReferenceKind::Borrow,
                                is_constant: false,
                                type_: Box::new(Type::Compound(Identifier::new("String")))
                            }
                        )
                    },
                ],
                output: Some(Type::Reference(
                    Reference {
                        kind: ReferenceKind::Borrow,
                        is_constant: true,
                        type_: Box::new(Type::Compound(Identifier::new("String")))
                    }
                ))
            }
        );
    }
}
