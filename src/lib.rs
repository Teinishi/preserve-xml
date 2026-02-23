pub mod attr_scanner;
pub mod content;
pub mod node;
pub mod scoped_reader;
pub mod utils;

pub use attr_scanner::{AttrSlot, Attributes};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Root {
        value: String,
        content: Content<Leaf>,
    }

    #[derive(Debug)]
    struct Leaf {
        value: i32,
        content: Content<()>,
    }

    impl ParseXml for Root {
        fn parse_start<'a, R: BufRead>(
            start: &BytesStart,
            builder: &mut ElementBuilder<'a, R>,
        ) -> quick_xml::Result<Option<WithMetadata<Self>>> {
            if start.name().as_ref() != b"root" {
                return Ok(None);
            }

            // 属性の処理
            let mut value = Default::default();
            let attributes = Attributes::new(start.attributes_raw(), |k, v| match k {
                b"value" => {
                    value = v;
                    true
                }
                _ => false,
            });

            // 子要素のパース
            let el = builder.build(
                attributes,
                |s, event| {
                    match event {
                        Event::Start(e) => {
                            if let Some(res) = Leaf::parse_start(&e, s)? {
                                return Ok(Some(res));
                            }
                        }
                        Event::Empty(e) => {
                            if let Some(res) = Leaf::parse_empty(&e, &(s.into()))? {
                                return Ok(Some(res));
                            }
                        }
                        _ => {}
                    }
                    Ok(None)
                },
                |content| Root { value, content },
            )?;

            Ok(Some(el))
        }

        fn parse_empty(
            start: &BytesStart,
            builder: &EmptyElementBuilder,
        ) -> quick_xml::Result<Option<WithMetadata<Self>>> {
            if start.name().as_ref() != b"root" {
                return Ok(None);
            }

            // 1. 属性の処理
            let mut value = Default::default();
            let attributes = Attributes::new(start.attributes_raw(), |k, v| match k {
                b"value" => {
                    value = v;
                    true
                }
                _ => false,
            });

            let el = builder.build(
                attributes,
                Root {
                    value,
                    content: Content::new(),
                },
            );

            Ok(Some(el))
        }
    }

    impl WriteXml for Root {
        fn write_xml<W: Write>(
            &self,
            attributes: &Attributes,
            writer: &mut W,
        ) -> std::io::Result<()> {
            write!(writer, "<root")?;

            attributes.write(writer, |k| match k {
                b"value" => Some((&self.value).into()),
                _ => None,
            })?;

            if self.content.is_empty() {
                writer.write_all(b"/>")?;
            } else {
                writer.write_all(b">")?;
                self.content.write_all(writer)?;
                writer.write_all(b"</root>")?;
            }

            Ok(())
        }
    }

    impl ParseXml for Leaf {
        fn parse_start<'a, R: BufRead>(
            start: &BytesStart,
            builder: &mut ElementBuilder<'a, R>,
        ) -> quick_xml::Result<Option<WithMetadata<Self>>> {
            if start.name().as_ref() != b"leaf" {
                return Ok(None);
            }

            // 属性の処理
            let mut value = Default::default();
            let attributes = Attributes::new(start.attributes_raw(), |k, v| match k {
                b"value" => {
                    value = v.parse().unwrap();
                    true
                }
                _ => false,
            });

            // 子要素のパース
            let el = builder.build(
                attributes,
                |_, _| Ok(None),
                |content| Leaf { value, content },
            )?;

            Ok(Some(el))
        }

        fn parse_empty(
            start: &BytesStart,
            builder: &EmptyElementBuilder,
        ) -> quick_xml::Result<Option<WithMetadata<Self>>> {
            if start.name().as_ref() != b"leaf" {
                return Ok(None);
            }

            // 属性の処理
            let mut value = Default::default();
            let attributes = Attributes::new(start.attributes_raw(), |k, v| match k {
                b"value" => {
                    value = v.parse().unwrap();
                    true
                }
                _ => false,
            });

            Ok(Some(builder.build(
                attributes,
                Leaf {
                    value,
                    content: Content::new(),
                },
            )))
        }
    }

    impl WriteXml for Leaf {
        fn write_xml<W: Write>(
            &self,
            attributes: &Attributes,
            writer: &mut W,
        ) -> std::io::Result<()> {
            write!(writer, "<leaf")?;

            attributes.write(writer, |k| match k {
                b"value" => Some(format!("{}", self.value).into()),
                _ => None,
            })?;

            if self.content.is_empty() {
                writer.write_all(b"/>")?;
            } else {
                writer.write_all(b">")?;
                self.content.write_all(writer)?;
                writer.write_all(b"</leaf>")?;
            }

            Ok(())
        }
    }

    #[test]
    fn case1() {
        let xml_str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<root value=\"foo\" uninterested_attribute=\"bar\"/>
<!-- hogehoge -->
";
        let tree = parse_str::<Root>(xml_str).unwrap();
        assert_eq!(xml_str, &tree.write_to_string());
    }

    #[test]
    fn case2() {
        let xml_str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<root value=\"multiple
    lines\" 01=\"bar\">
    <leaf value=\"123\" />
    <leaf
        value = \"456\">hoge</leaf>
    <uninterested value=\"789\">
        <leaf />
    </uninterested>
</root>
";
        let tree = parse_str::<Root>(xml_str).unwrap();
        assert_eq!(xml_str, &tree.write_to_string());
    }
}
