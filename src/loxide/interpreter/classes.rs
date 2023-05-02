use std::fmt;

#[derive(Clone)]
pub struct Class {
    pub name: String,
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}
