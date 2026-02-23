pub mod content;
pub mod node;
pub mod scoped_reader;

pub use content::{Content, ParseContent, SerializeContent};
pub use node::*;
pub use scoped_reader::{ElementBuilder, EmptyElementBuilder};

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
    fn write_xml<W: Write>(&self, attributes: &[AttrSlot], writer: &mut W) -> std::io::Result<()>;
}

impl WriteXml for () {
    fn write_xml<W: Write>(
        &self,
        _attributes: &[AttrSlot],
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
            let mut attributes = Vec::new();
            let mut value = String::new();
            for attr in start.attributes().flatten() {
                let key = attr.key.as_ref().to_vec();
                if attr.key.as_ref() == b"value" {
                    // 定義済み属性
                    attributes.push(AttrSlot::Defined(key));
                    value = String::from_utf8_lossy(&attr.value).into_owned();
                } else {
                    // 未定義属性
                    attributes.push(AttrSlot::Raw(key, attr.value.as_ref().to_vec()));
                }
            }

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
            let mut attributes = Vec::new();
            let mut value = String::new();
            for attr in start.attributes().flatten() {
                let key = attr.key.as_ref().to_vec();
                if attr.key.as_ref() == b"value" {
                    // 定義済み属性
                    attributes.push(AttrSlot::Defined(key));
                    value = String::from_utf8_lossy(&attr.value).into_owned();
                } else {
                    // 未定義属性
                    attributes.push(AttrSlot::Raw(key, attr.value.as_ref().to_vec()));
                }
            }

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
            attributes: &[AttrSlot],
            writer: &mut W,
        ) -> std::io::Result<()> {
            write!(writer, "<root")?;

            for attr in attributes {
                match attr {
                    AttrSlot::Defined(name) => match &name[..] {
                        b"value" => {
                            write!(writer, " value=\"{}\"", self.value)?;
                        }
                        _ => {
                            panic!("Unknown attribute");
                        }
                    },
                    AttrSlot::Raw(key, value) => {
                        writer.write_all(b" ")?;
                        writer.write_all(key)?;
                        writer.write_all(b"=\"")?;
                        writer.write_all(value)?;
                        writer.write_all(b"\"")?;
                    }
                }
            }

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
            let mut attributes = Vec::new();
            let mut value = 0;
            for attr in start.attributes().flatten() {
                let key = attr.key.as_ref().to_vec();
                if attr.key.as_ref() == b"value" {
                    // 定義済み属性
                    attributes.push(AttrSlot::Defined(key));
                    value = std::str::from_utf8(&attr.value)
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap();
                } else {
                    // 未定義属性
                    attributes.push(AttrSlot::Raw(key, attr.value.as_ref().to_vec()));
                }
            }

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

            // 1. 属性の処理
            let mut attributes = Vec::new();
            let mut value = 0;
            for attr in start.attributes().flatten() {
                let key = attr.key.as_ref().to_vec();
                if attr.key.as_ref() == b"value" {
                    // 定義済み属性
                    attributes.push(AttrSlot::Defined(key));
                    value = std::str::from_utf8(&attr.value)
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap();
                } else {
                    // 未定義属性
                    attributes.push(AttrSlot::Raw(key, attr.value.as_ref().to_vec()));
                }
            }

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
            attributes: &[AttrSlot],
            writer: &mut W,
        ) -> std::io::Result<()> {
            write!(writer, "<leaf")?;

            for attr in attributes {
                match attr {
                    AttrSlot::Defined(name) => match &name[..] {
                        b"value" => {
                            write!(writer, " value=\"{}\"", self.value)?; // todo: エスケープ
                        }
                        _ => {
                            panic!("Unknown attribute");
                        }
                    },
                    AttrSlot::Raw(key, value) => {
                        writer.write_all(b" ")?;
                        writer.write_all(key)?;
                        writer.write_all(b"=\"")?;
                        writer.write_all(value)?;
                        writer.write_all(b"\"")?;
                    }
                }
            }

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
    <leaf value=\"456\">hoge</leaf>
    <uninterested value=\"789\">
        <leaf />
    </uninterested>
</root>
";
        let tree = parse_str::<Root>(xml_str).unwrap();
        assert_eq!(xml_str, &tree.write_to_string());
    }
}
