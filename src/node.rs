use super::{Attributes, utils::debug_utf8};

use std::fmt::Debug;

#[derive(Clone, Debug)]
#[non_exhaustive] // 外部から WithMetadata { .. } で生成できないように
pub struct WithMetadata<T> {
    pub attributes: Attributes,
    pub element: T,
}

impl<T> WithMetadata<T> {
    pub(crate) fn new(attributes: Attributes, element: T) -> Self {
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
        #[expect(dead_code)]
        #[derive(Debug)]
        enum Node<'a, T> {
            Typed(&'a WithMetadata<T>),
            Raw(&'a str),
        }
        let tmp = match self {
            Self::Typed(data) => Node::Typed(data),
            Self::Raw(data) => Node::Raw(debug_utf8(data)),
        };
        tmp.fmt(f)
    }
}
