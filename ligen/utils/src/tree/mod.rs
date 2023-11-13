pub mod identifier;
pub mod path;

use std::{collections::HashMap, borrow::Borrow, fmt::Display};

pub use identifier::*;
pub use path::*;

use crate::prelude::*;

pub trait IsTree: HasIdentifier {
    fn is(&self, identifier: impl PartialEq<Self::Identifier>) -> bool {
        identifier.eq(self.identifier())
    }

    fn add_branch(&mut self, _child: impl Into<Self>) -> &mut Self where Self: Sized {
        self
    }

    fn remove_branch(&mut self, _identifier: &Self::Identifier) {}

    fn branches<'a>(&'a self) -> Box<dyn Iterator<Item = &Self> + 'a>;
    fn branches_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Self> + 'a>;

    fn branch<K>(&mut self, key: K) -> &mut Self
    where K: Into<Self::Identifier>,
          Self::Identifier: Borrow<Self::Identifier>,
          Self: From<Self::Identifier>
    {
        // FIXME: This is a workaround for
        //  https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions
        //  Fix it when the borrow checker is fixed.
        let myself = unsafe { &mut *(self as *mut Self) };
        let key = key.into();
        if let Some(value) = myself.get_mut(key.clone()) {
            value
        } else {
            self.add_branch(Self::from(key))
        }
    }

    fn get<K>(&self, key: K) -> Option<&Self>
    where K: Into<Self::Identifier>, Self::Identifier: Borrow<Self::Identifier>;
    
    fn get_mut<K>(&mut self, key: K) -> Option<&mut Self>
    where K: Into<Self::Identifier>, Self::Identifier: Borrow<Self::Identifier>;
    
    fn path_get<'a, K>(&'a self, path: impl IntoIterator<Item = K>) -> Option<&Self>
    where K: Into<Self::Identifier>, Self::Identifier: Borrow<Self::Identifier>,
    Self::Identifier: Display
    {
        let mut path = path.into_iter();
        if let Some(segment) = path.next() {
            let segment = segment.into();
            self
                .get(segment)
                .and_then(|branch|
                    branch.path_get(path)
                )
        } else {
            Some(self)
        }
    }
}

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Tree<Value>
where Value: HasIdentifier
{
    #[shrinkwrap(main_field)]
    pub value: Value,
    pub children: HashMap<Value::Identifier, Tree<Value>>
}

impl<Value> HasIdentifier for Tree<Value>
where Value: HasIdentifier
{
    type Identifier = Value::Identifier;
    fn identifier(&self) -> &Self::Identifier {
        self.value.identifier()
    }
}

impl<Value> IsTree for Tree<Value>
where Value: HasIdentifier
{
    fn add_branch(&mut self, child: impl Into<Self>) -> &mut Self
    where Self: Sized
    {
        let child = child.into();
        self.children
            .entry(child.identifier().clone())
            .or_insert(child)
    }

    fn branches<'a>(&'a self) -> Box<dyn Iterator<Item = &Self> + 'a> {
        Box::new(self.children.values())
    }

    fn branches_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Self> + 'a> {
        Box::new(self.children.values_mut())        
    }

    fn get<K>(&self, key: K) -> Option<&Self>
    where K: Into<Self::Identifier>, Self::Identifier: Borrow<Self::Identifier>{
        let key = key.into();
        let key = key.borrow();
        self
            .children
            .get(key)
    }

    fn get_mut<K>(&mut self, key: K) -> Option<&mut Self>
    where K: Into<Self::Identifier>, Self::Identifier: Borrow<Self::Identifier>{
        let key = key.into();
        let key = key.borrow();
        self
            .children
            .get_mut(key)
    }
}

impl<Value> From<Value> for Tree<Value>
where Value: HasIdentifier
{
    fn from(value: Value) -> Self {
        let children = Default::default();
        Self { value, children }
    }
}

#[cfg(test)]
mod test {
    use super::*;


    impl HasIdentifier for usize {
        type Identifier = usize;
        fn identifier(&self) -> &Self::Identifier {
            self
        }
    }
    
    pub struct Module {
        identifier: String
    }
    
    impl Module {
        pub fn format(&self) -> String {
            format!("[{}]", self.identifier)
        }
    }
    
    impl<S: Into<String>> From<S> for Module {
        fn from(name: S) -> Self {
            let name = name.into();
            Self { identifier: name }
        }
    }
    
    impl HasIdentifier for Module {
        type Identifier = String;
        fn identifier(&self) -> &Self::Identifier {
            &self.identifier
        }
    }

    fn create() -> Tree<Module> {
        let tree = Tree::from(5);
        assert!(tree.is(5 as usize));

        let mut root = Tree::from(Module::from("root"));
        assert!(root.is("root"));
        assert_eq!(root.format(), "[root]");

        let branch = root.add_branch(Module::from("branch"));
        assert!(branch.is("branch"));
        assert_eq!(branch.format(), "[branch]");
        
        let leaf = branch.add_branch(Module::from("leaf"));
        assert!(leaf.is("leaf"));
        assert_eq!(leaf.format(), "[leaf]");

        root
    }

    #[test]
    fn creation() {
        create();
    }

    #[test]
    fn get() {
        let root = create();
        assert!(root.is("root"));
        assert_eq!(root.format(), "[root]");

        let branch = root.get("branch").unwrap();
        assert!(branch.is("branch"));
        assert_eq!(branch.format(), "[branch]");

        let leaf = branch.get("leaf").unwrap();
        assert!(leaf.is("leaf"));
        assert_eq!(leaf.format(), "[leaf]");
    }

    #[test]
    fn get_from_path() {
        let root = create();
        let jose = root.path_get::<&str>([]).unwrap();
        assert!(jose.is("root"));
        assert_eq!(jose.format(), "[root]");

        assert!(jose.path_get(["none"]).is_none());
        assert!(jose.path_get(["branch", "fruit"]).is_none());

        let danilo = jose.path_get(["branch"]).unwrap();
        assert!(danilo.is("branch"));
        assert_eq!(danilo.format(), "[branch]");

        let joaquim = jose.path_get(["branch", "leaf"]).unwrap();
        assert!(joaquim.is("leaf"));
        assert_eq!(joaquim.format(), "[leaf]");
    }

}
