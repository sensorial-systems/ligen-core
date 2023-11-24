use std::rc::Rc;

use crate::{IsIdentifier, HasIdentifier, Path, Identifier, IsTree};

pub struct Visitor<'a, Value>
where Value: HasIdentifier
{
    pub parent: Option<Rc<Visitor<'a, Value>>>,
    pub value: &'a Value,
    pub path: Path<'a, Value::Identifier>
}

impl<'a, Value> Visitor<'a, Value>
where Value: HasIdentifier
{
    pub fn new(value: &'a Value, parent: Option<Rc<Visitor<'a, Value>>>, path: Path<'a, Value::Identifier>) -> Rc<Self> {
        Rc::new(Self { value, parent, path })
    }

    pub fn parent(self: &Rc<Self>) -> Option<Rc<Self>> {
        self.parent.clone()
    }

    pub fn child(self: &Rc<Self>, value: &'a Value) -> Rc<Self>
    {
        let path = self.path.join(value.identifier().clone());
        let child = Self::new(value, Some(self.clone()), path);
        child
    }

    pub fn root(self: &Rc<Self>) -> Rc<Self> {
        self.parent
            .as_ref()
            .map(|parent| parent.root())
            .unwrap_or(self.clone())
    }

    pub fn relative<K>(self: &Rc<Self>, path: impl IntoIterator<Item = K>) -> Option<Rc<Self>>
    where K: Into<Value::Identifier>,
        Value: IsTree
    {
        let mut path = path.into_iter();
        if let Some(segment) = path.next() {
            let segment = segment.into();
            match segment.kind() {
                Identifier::Root => Some(self.root()),
                Identifier::Self_ => self.relative(path),
                Identifier::Super => self
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.relative(path)),
                Identifier::Other(segment) => self
                    .value
                    .get(segment.clone())
                    .and_then(|branch|
                        self.child(branch)
                            .relative(path)
                    )
            }
        } else {
            Some(self.clone())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Value {
        identifier: String
    }

    impl HasIdentifier for Value {
        type Identifier = String;
        fn identifier(&self) -> &Self::Identifier {
            &self.identifier
        }
    }

    impl Value {
        fn new(identifier: impl Into<String>) -> Self {
            Self {
                identifier: identifier.into()
            }
        }
    }

    #[test]
    fn visitor() {
        let root = Value::new("root");
        let root = Visitor::new(&root, None, Path::default());
        let branch = Value::new("branch");
        let branch = root.child(&branch);
        let leaf = Value::new("leaf");
        let leaf = branch.child(&leaf);
        assert_eq!(root.root().value.identifier(), "root");
        assert_eq!(branch.root().value.identifier(), "root");
        assert_eq!(branch.parent().unwrap().value.identifier(), "root");
        assert_eq!(leaf.root().value.identifier(), "root");
        assert_eq!(leaf.parent().unwrap().value.identifier(), "branch");
        assert_eq!(leaf.parent().unwrap().parent().unwrap().value.identifier(), "root");
    }

    mod new_visitor {
        use std::rc::Rc;

        pub struct Visitor<Parent, Value>
        where Parent: HasParent
        {
            parent: Option<Rc<Visitor<Parent::Parent, Parent>>>,
            value: Value
        }

        impl<Parent, Value> Visitor<Parent, Value>
        where Parent: HasParent, Value: HasParent<Parent = Parent>
        {
            pub fn new(parent: Option<Rc<Visitor<Parent::Parent, Parent>>>, value: Value) -> Rc<Self> {
                Rc::new(Self { parent, value })
            }

            pub fn child<Child>(self: &Rc<Self>, child: Child) -> Rc<Visitor<Value, Child>>
            where Child: HasParent<Parent = Value>,

            {
                Visitor::new(Some(self.clone().into()), child)
            }

            pub fn parent(self: &Rc<Self>) -> Option<Rc<Visitor<Parent::Parent, Parent>>> {
                self.parent.clone()
            }
        }

        trait HasParent {
            type Parent: HasParent;
        }

        impl HasParent for () {
            type Parent = ();
        }

        impl HasParent for &Library {
            type Parent = ();
        }

        enum ModuleParent<'a> {
            Library(&'a Library),
            Module(&'a Module)
        }

        impl<'a> HasParent for ModuleParent<'a> {
            type Parent = ModuleParent<'a>;
        }

        impl<'a> HasParent for &'a Module {
            type Parent = ModuleParent<'a>;
        }

        struct Library {
            name: String,
            root_module: Module
        }

        struct Module {
            name: String,
            modules: Vec<Module>
        }

        #[test]
        fn new_visitor() {
            let library = Library {
                name: "library".into(),
                root_module: Module {
                    name: "root".into(),
                    modules: vec![
                        Module {
                            name: "sub".into(),
                            modules: vec![]
                        }
                    ]
                }
            };

            let library = Visitor::<(), _>::new(None, &library);
            assert_eq!(library.value.name, "library");
            let root_module = library.child(&library.value.root_module);
            assert_eq!(root_module.value.name, "root");
            let sub_module = root_module.child(&root_module.value.modules[0]);
            // assert_eq!(sub_module.value.name, "sub");
            // assert_eq!(sub_module.parent().unwrap().name, "root");
            // assert_eq!(sub_module.parent().unwrap().parent().unwrap().name, "library");
        }
    }
}
