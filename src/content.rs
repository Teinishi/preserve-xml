use super::{Node, WithMetadata};

pub trait ParseContent<T> {
    fn new() -> Self;
    fn push(&mut self, value: WithMetadata<T>);
    fn push_raw(&mut self, value: Vec<u8>);
}

// 任意の数の T と RawData を持てる
#[derive(Clone, Debug)]
pub struct Content<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Default for Content<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<T> Content<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Typed(wm) => Some(&wm.element),
            Node::Raw(_) => None,
        })
    }

    pub fn all_nodes(&self) -> &[Node<T>] {
        &self.nodes
    }
}

impl<T> ParseContent<T> for Content<T> {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, value: WithMetadata<T>) {
        self.nodes.push(Node::Typed(value));
    }

    fn push_raw(&mut self, value: Vec<u8>) {
        self.nodes.push(Node::Raw(value));
    }
}

// todo: RawData のみを持つ
// todo: 何も持たない
// todo: 1個のみ T を持つ
