//! Function parameter.

use crate::prelude::*;
use crate::{Identifier, Reference, Type, ReferenceKind, Mutability, Parameter};

impl TryFrom<syn::FnArg> for Parameter {
    type Error = Error;

    fn try_from(fn_arg: syn::FnArg) -> Result<Self> {
        match fn_arg {
            syn::FnArg::Typed(syn::PatType { pat, ty, attrs, .. }) => {
                if let syn::Pat::Ident(syn::PatIdent { ident, .. }) = *pat {
                    Ok(Self {
                        attributes: attrs.try_into()?,
                        identifier: ident.into(),
                        type_: Type::try_from(*ty).expect("Failed to convert from Type"),
                    })
                } else {
                    Err(Error::Message("Identifier not found".into()))
                }
            }
            // TODO: Implement conversion for syn::Receiver. <- What does it mean?
            syn::FnArg::Receiver(syn::Receiver {
                                attrs,
                                reference,
                                mutability,
                                ..
                            }) => {
                let attributes = attrs.try_into()?;
                let identifier = Identifier::new("self").into();
                let type_ = reference
                    .map(|_| {
                        let kind = ReferenceKind::Borrow;
                        let mutability = if mutability.is_none() { Mutability::Constant } else { Mutability::Mutable };
                        let type_ = Box::new(Type::from(Identifier::new("Self")));
                        Type::Reference(Reference { kind, mutability, type_ })
                    })
                    .unwrap_or_else(|| Type::from(Identifier::new("Self")));
                Ok(Self { attributes, identifier, type_ })
            },
        }
    }
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.identifier.to_token_stream();
        let typ = self.type_.to_token_stream();
        tokens.append_all(quote! {#ident: #typ})
    }
}

#[cfg(test)]
mod test {

    use std::convert::TryFrom;

    use super::Parameter;
    use crate::{Atomic, Identifier, Integer, Reference, Type, ReferenceKind, Attribute, Mutability};
    use quote::quote;
    use syn::{parse_quote::parse, FnArg};

    #[test]
    fn parameter_atomic() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {
                #[attribute] integer: i32
            })).expect("Returned Error"),
            Parameter {
                attributes: Attribute::Group("attribute".into(), Default::default()).into(),
                identifier: Identifier::new("integer"),
                type_: Type::Atomic(Atomic::Integer(Integer::I32))
            }
        );
    }

    #[test]
    fn parameter_compound() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {name: String})).expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("name"),
                type_: Type::Compound(Identifier::new("String").into(), Default::default())
            }
        );
    }

    #[test]
    fn parameter_borrow_constant() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {name: &String})).expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("name"),
                type_: Type::Reference(
                    Reference {
                        kind: ReferenceKind::Borrow,
                        mutability: Mutability::Constant,
                        type_: Box::new(Type::Compound(Identifier::new("String").into(), Default::default()))
                    }
                )

            }
        );
    }

    #[test]
    fn parameter_borrow_mutable() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {name: &mut String}))
                .expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("name"),
                type_: Type::Reference(
                    Reference {
                        kind: ReferenceKind::Borrow,
                        mutability: Mutability::Mutable,
                        type_: Box::new(Type::Compound(Identifier::new("String").into(), Default::default()))
                    }
                )

            }
        );
    }

    #[test]
    fn parameter_pointer_constant() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {name: *const String}))
                .expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("name"),
                type_: Type::Reference(
                    Reference {
                        kind: ReferenceKind::Pointer,
                        mutability: Mutability::Constant,
                        type_: Box::new(Type::Compound(Identifier::new("String").into(), Default::default()))
                    }
                )

            }
        );
    }

    #[test]
    fn parameter_pointer_mutable() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {name: *mut String}))
                .expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("name"),
                type_: Type::Reference(
                    Reference {
                        kind: ReferenceKind::Pointer,
                        mutability: Mutability::Mutable,
                        type_: Box::new(Type::Compound(Identifier::new("String").into(), Default::default()))
                    }
                )
            }
        );
    }

    #[test]
    fn parameter_receiver() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {self})).expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("self").into(),
                type_: Type::Compound(Identifier::new("Self").into(), Default::default())
            }
        );
    }

    #[test]
    fn parameter_receiver_reference() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {&self})).expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("self").into(),
                type_: Type::Reference(
                    Reference {
                        kind: ReferenceKind::Borrow,
                        mutability: Mutability::Constant,
                        type_: Box::new(Type::Compound(Identifier::new("Self").into(), Default::default()))
                    }
                )
            }
        );
    }

    #[test]
    fn parameter_receiver_mutable() {
        assert_eq!(
            Parameter::try_from(parse::<FnArg>(quote! {&mut self})).expect("Returned Error"),
            Parameter {
                attributes: Default::default(),
                identifier: Identifier::new("self").into(),
                type_: Type::Reference(
                    Reference {
                        kind: ReferenceKind::Borrow,
                        mutability: Mutability::Mutable,
                        type_: Box::new(Type::Compound(Identifier::new("Self").into(), Default::default()))
                    }
                )
            }
        );
    }
}
