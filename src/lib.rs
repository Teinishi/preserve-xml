pub mod attributes;
pub mod content;
pub mod node;
pub mod scoped_reader;
pub mod utils;

pub use attributes::{AttrSlot, Attributes};
pub use content::{Content, ParseContent, SerializeContent};
pub use node::*;
pub use scoped_reader::{ElementBuilder, EmptyElementBuilder};
pub use utils::{escape_xml, unescape_xml};

use quick_xml::Reader;
pub use quick_xml::events::{BytesStart, Event};
use std::io::{BufRead, Write};

pub trait ParseXml: Sized {
    fn parse_start<'a, R: BufRead>(
        start: &BytesStart,
        builder: &mut ElementBuilder<'a, R>,
    ) -> quick_xml::Result<Option<WithMetadata<Self>>>;
    fn parse_empty(
        start: &BytesStart,
        builder: &EmptyElementBuilder,
    ) -> quick_xml::Result<Option<WithMetadata<Self>>>;
}

pub trait WriteXml {
    fn write_xml<W: Write>(&self, attributes: &Attributes, writer: &mut W) -> std::io::Result<()>;
}

impl WriteXml for () {
    fn write_xml<W: Write>(
        &self,
        _attributes: &Attributes,
        _writer: &mut W,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn parse<R: BufRead, T: ParseXml>(reader: &mut Reader<R>) -> quick_xml::Result<Content<T>> {
    let mut scope = ElementBuilder::new(reader);

    let content = scope.parse_content(|s, event| {
        match event {
            Event::Start(e) => {
                if let Some(data) = T::parse_start(&e, s)? {
                    return Ok(Some(data));
                }
            }
            Event::Empty(e) => {
                if let Some(data) = T::parse_empty(&e, &s.into())? {
                    return Ok(Some(data));
                }
            }
            _ => {}
        }
        Ok(None)
    })?;

    Ok(content)
}

pub fn parse_str<T: ParseXml>(s: &str) -> quick_xml::Result<Content<T>> {
    let mut reader = quick_xml::Reader::from_str(s);
    parse(&mut reader)
}
