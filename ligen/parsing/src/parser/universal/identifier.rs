use crate::prelude::*;

use ligen_ir::Identifier;
use crate::parser::Parser;

#[derive(Default)]
pub struct IdentifierParser;

impl Parser<String> for IdentifierParser {
    type Output = Identifier;
    fn parse(&self, input: String) -> Result<Self::Output> {
        self.parse(input.as_str())
    }
}

impl Parser<&str> for IdentifierParser {
    type Output = Identifier;
    fn parse(&self, input: &str) -> Result<Self::Output> {
        syn::parse_str::<syn::Ident>(input)
            .map_err(|e| Error::Message(format!("Failed to parse identifier: {:?}", e)))
            .and_then(|ident| self.parse(ident))
    }
}

impl Parser<syn::Ident> for IdentifierParser {
    type Output = Identifier;
    fn parse(&self, ident: syn::Ident) -> Result<Self::Output> {
        let name = ident.to_string();
        Ok(Self::Output { name })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert::*;
    use ligen_ir::identifier::mock;

    #[test]
    fn identifier() -> Result<()> {
        assert_eq(IdentifierParser, mock::identifier(), "identifier")
    }
}
