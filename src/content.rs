use super::{Node, WithMetadata, WriteXml};

use std::io::Write;

pub trait ParseContent<T> {
    fn new() -> Self;
    fn push(&mut self, value: WithMetadata<T>);
    fn push_raw(&mut self, value: Vec<u8>);
}

pub trait SerializeContent<T> {
    fn is_empty(&self) -> bool;
    fn write_all<W: Write>(&self, writer: &mut W) -> std::io::Result<()>
    where
        T: WriteXml;
    fn write_to_string(&self) -> String
    where
        T: WriteXml,
    {
        let mut bytes = Vec::new();
        self.write_all(&mut bytes).unwrap();
        String::from_utf8(bytes).unwrap()
    }
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

impl<T> SerializeContent<T> for Content<T> {
    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn write_all<W: Write>(&self, writer: &mut W) -> std::io::Result<()>
    where
        T: WriteXml,
    {
        for node in &self.nodes {
            match node {
                Node::Raw(bytes) => writer.write_all(bytes)?,
                Node::Typed(data) => data.element.write_xml(&data.attributes, writer)?,
            }
        }

        Ok(())
    }
}

// todo: RawData のみを持つ Content
// todo: 何も持たない Content
// todo: 1個のみ T を持つ Content
