use std::fmt::Debug;

fn debug_utf8(bytes: &[u8]) -> &str {
    str::from_utf8(bytes).unwrap_or("[non-utf8]")
}

#[derive(Clone, PartialEq, Eq)]
pub enum AttrSlot {
    Defined(Vec<u8>), // todo: 復元をどうするか考える
    Raw(Vec<u8>, Vec<u8>),
}

impl Debug for AttrSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined(name) => write!(f, "Defined({:?})", debug_utf8(name)),
            Self::Raw(name, value) => {
                write!(f, "Raw({:?}, {:?})", debug_utf8(name), debug_utf8(value))
            }
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive] // 外部から WithMetadata { .. } で生成できないように
pub struct WithMetadata<T> {
    pub attributes: Vec<AttrSlot>,
    pub element: T,
}

impl<T> WithMetadata<T> {
    pub(crate) fn new(attributes: Vec<AttrSlot>, element: T) -> Self {
        Self {
            attributes,
            element,
        }
    }
}

#[derive(Clone)]
pub enum Node<T> {
    Typed(WithMetadata<T>),
    Raw(Vec<u8>),
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Typed(data) => write!(f, "Typed({:?})", data),
            Self::Raw(data) => {
                write!(f, "Raw({:?})", debug_utf8(data))
            }
        }
    }
}
