use crate::prelude::*;

use crate::{Attributes, Identifier, Mutability, Parameter, Type, Visibility};

pub mod parameter;

/// Async structure.
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Async;

/// Method structure.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Method {
    pub mutability: Mutability,
    /// Method owner.
    pub owner: Type
}

/// Function structure.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Attributes field.
    pub attributes: Attributes,
    /// Visibility field.
    pub visibility: Visibility,
    // FIXME: Rework it as owner: Option<Owner>?
    /// Method field.
    pub method: Option<Method>,
    /// Asyncness field.
    pub asyncness: Option<Async>,
    /// Identifier field.
    pub identifier: Identifier,
    /// Inputs field.
    pub inputs: Vec<Parameter>,
    /// Output field.
    pub output: Option<Type>,
}
