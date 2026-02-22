use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::{fmt::Debug, io::BufRead};

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct XmlMeta {
    pub attributes: Vec<AttrSlot>,
}

#[derive(Clone, Debug)]
pub struct WithMetadata<T> {
    pub meta: XmlMeta,
    pub inner: T,
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
    pub fn push(&mut self, value: WithMetadata<T>) {
        self.nodes.push(Node::Typed(value));
    }

    pub fn push_raw(&mut self, value: Vec<u8>) {
        self.nodes.push(Node::Raw(value));
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Typed(wm) => Some(&wm.inner),
            Node::Raw(_) => None,
        })
    }

    pub fn all_nodes(&self) -> &[Node<T>] {
        &self.nodes
    }
}

pub trait ParseXml: Sized {
    fn parse_start<R: BufRead>(
        start: &BytesStart,
        reader: &mut Reader<R>,
    ) -> Option<quick_xml::Result<WithMetadata<Self>>>;
    fn parse_empty(start: &BytesStart) -> Option<quick_xml::Result<WithMetadata<Self>>>;
}

/*fn capture_raw_start<R: BufRead>(start: &BytesStart, reader: &mut Reader<R>) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);

    // 開始タグを書き込む
    writer.write_event(Event::Start(start.clone())).unwrap();

    let mut depth = 1;
    let mut temp_buf = Vec::new();

    while depth > 0 {
        match reader.read_event_into(&mut temp_buf).unwrap() {
            Event::Start(e) => {
                depth += 1;
                writer.write_event(Event::Start(e)).unwrap();
            }
            Event::End(e) => {
                depth -= 1;
                writer.write_event(Event::End(e)).unwrap();
            }
            Event::Eof => break,
            e => {
                writer.write_event(e).unwrap();
            }
        }
        temp_buf.clear();
    }
    buf
}

fn capture_raw_empty(start: &BytesStart) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut writer = quick_xml::Writer::new(&mut buf);

    // タグを書き込む
    writer.write_event(Event::Empty(start.clone())).unwrap();

    buf
}*/

fn read_until_end<R: BufRead>(reader: &mut Reader<R>, buf: &mut Vec<u8>) -> quick_xml::Result<()> {
    let mut stack = Vec::new();
    loop {
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                stack.push(e.name().as_ref().to_vec());
            }
            Event::End(e) => {
                if stack.is_empty() {
                    return Ok(());
                } else if stack.last().is_some_and(|s| s == e.name().as_ref()) {
                    stack.pop();
                }
            }
            Event::Eof => {
                panic!("Closing tag not found");
            }
            _ => {}
        }
    }
}

pub struct ScopedReader<'a, R> {
    reader: &'a mut Reader<R>,
    stack: Vec<Vec<u8>>,
}

impl<'a, R> ScopedReader<'a, R> {
    pub fn new(reader: &'a mut Reader<R>) -> Self {
        Self {
            reader,
            stack: Vec::new(),
        }
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn read_event_into<'b>(&'a mut self, buf: &'b mut Vec<u8>) -> quick_xml::Result<Event<'b>>
    where
        R: BufRead,
    {
        let event = self.reader.read_event_into(buf)?;

        match &event {
            Event::Start(e) => {
                self.stack.push(e.name().as_ref().to_vec());
            }
            Event::End(e) => {
                if let Some(s) = self.stack.last() {
                    if e.name().as_ref() == s {
                        self.stack.pop();
                    }
                } else {
                    return Ok(Event::Eof);
                }
            }
            _ => {}
        }

        Ok(event)
    }
}

pub fn parse_xml_str<T: ParseXml>(s: &str) -> quick_xml::Result<Content<T>> {
    let mut content = Content::default();
    let mut reader = quick_xml::Reader::from_str(s);
    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();

    loop {
        match reader.read_event_into(&mut buf2)? {
            Event::Start(e) => {
                if let Some(data) = T::parse_start(&e, &mut reader) {
                    content.push_raw(buf1);
                    content.push(data?);
                    break;
                }
            }
            Event::Empty(e) => {
                if let Some(data) = T::parse_empty(&e) {
                    content.push_raw(buf1);
                    content.push(data?);
                    break;
                }
            }
            Event::Eof => {
                content.push_raw(buf1);
                break;
            }
            _ => {}
        }
        buf1.append(&mut buf2);
    }

    buf2.clear();
    loop {
        if reader.read_event_into(&mut buf2)? == Event::Eof {
            content.push_raw(buf2);
            break;
        }
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
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
            fn parse_start<R: BufRead>(
                start: &BytesStart,
                reader: &mut Reader<R>,
            ) -> Option<quick_xml::Result<WithMetadata<Self>>> {
                if start.name().as_ref() != b"root" {
                    return None;
                }

                let mut attributes = Vec::new();
                let mut value = String::new();

                // 1. 属性の処理
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

                // 2. 子要素のループ
                let mut content = Content::default();
                let mut buf1 = Vec::new();
                let mut buf2 = Vec::new();
                loop {
                    match reader.read_event_into(&mut buf2) {
                        Ok(Event::Start(e)) => {
                            match Leaf::parse_start(&e, reader) {
                                Some(Ok(data)) => {
                                    // 知っているタグならパース
                                    content.push_raw(buf1.drain(..).collect());
                                    content.push(data);
                                    buf2.clear();
                                }
                                Some(Err(e)) => return Some(Err(e)),
                                None => {
                                    // 未知のタグなら閉じるまで進める
                                    if let Err(e) = read_until_end(reader, &mut buf2) {
                                        return Some(Err(e));
                                    }
                                }
                            }
                        }
                        Ok(Event::Empty(e)) => {
                            match Leaf::parse_empty(&e) {
                                Some(Ok(data)) => {
                                    // 知っているタグならパース
                                    content.push_raw(buf1.drain(..).collect());
                                    content.push(data);
                                    buf2.clear();
                                }
                                Some(Err(e)) => return Some(Err(e)),
                                None => {}
                            }
                        }
                        Ok(Event::End(_)) | Ok(Event::Eof) => {
                            content.push_raw(buf1);
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => return Some(Err(e)),
                    }
                    buf1.append(&mut buf2);
                }

                Some(Ok(WithMetadata {
                    meta: XmlMeta { attributes },
                    inner: Root { value, content },
                }))
            }

            fn parse_empty(start: &BytesStart) -> Option<quick_xml::Result<WithMetadata<Self>>> {
                if start.name().as_ref() != b"root" {
                    return None;
                }

                let mut attributes = Vec::new();
                let mut value = String::new();

                // 1. 属性の処理
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

                Some(Ok(WithMetadata {
                    meta: XmlMeta { attributes },
                    inner: Root {
                        value,
                        content: Content::default(),
                    },
                }))
            }
        }

        impl ParseXml for Leaf {
            fn parse_start<R: BufRead>(
                start: &BytesStart,
                reader: &mut Reader<R>,
            ) -> Option<quick_xml::Result<WithMetadata<Self>>> {
                if start.name().as_ref() != b"leaf" {
                    return None;
                }

                let mut attributes = Vec::new();
                let mut value = 0;

                // 1. 属性の処理
                for attr in start.attributes().flatten() {
                    let key = attr.key.as_ref().to_vec();
                    if attr.key.as_ref() == b"value" {
                        // 定義済み属性
                        attributes.push(AttrSlot::Defined(key));
                        value = std::str::from_utf8(attr.value.as_ref())
                            .ok()
                            .and_then(|v| v.parse().ok())
                            .unwrap();
                    } else {
                        // 未定義属性
                        attributes.push(AttrSlot::Raw(key, attr.value.as_ref().to_vec()));
                    }
                }

                // 2. 子要素のループ
                let mut content = Content::default();
                let mut buf1 = Vec::new();
                let mut buf2 = Vec::new();
                loop {
                    match reader.read_event_into(&mut buf2) {
                        Ok(Event::Start(e)) => {
                            // 閉じるまで進める
                            if let Err(e) = read_until_end(reader, &mut buf2) {
                                return Some(Err(e));
                            }
                        }
                        Ok(Event::End(_)) | Ok(Event::Eof) => {
                            content.push_raw(buf1);
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            return Some(Err(e));
                        }
                    }
                    buf1.append(&mut buf2);
                }

                Some(Ok(WithMetadata {
                    meta: XmlMeta { attributes },
                    inner: Leaf { value, content },
                }))
            }

            fn parse_empty(start: &BytesStart) -> Option<quick_xml::Result<WithMetadata<Self>>> {
                if start.name().as_ref() != b"leaf" {
                    return None;
                }

                let mut attributes = Vec::new();
                let mut value = 0;

                // 1. 属性の処理
                for attr in start.attributes().flatten() {
                    let key = attr.key.as_ref().to_vec();
                    if attr.key.as_ref() == b"value" {
                        attributes.push(AttrSlot::Defined(key));
                        value = std::str::from_utf8(attr.value.as_ref())
                            .ok()
                            .and_then(|v| v.parse().ok())
                            .unwrap();
                    } else {
                        attributes.push(AttrSlot::Raw(key, attr.value.as_ref().to_vec()));
                    }
                }

                Some(Ok(WithMetadata {
                    meta: XmlMeta { attributes },
                    inner: Leaf {
                        value,
                        content: Content::default(),
                    },
                }))
            }
        }

        let doc1 = parse_xml_str::<Root>(
            "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<root value=\"foo\" uninterested_attribute=\"bar\"/>
",
        )
        .unwrap();
        println!("{:?}", doc1);

        /*let doc2 = parse_xml_str::<Root>(
            "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<root value=\"foo\" uninterested_attribute=\"bar\">
    <leaf value=\"123\" />
    <leaf value=\"456\" />
    <uninterested value=\"789\" />
</root>
",
        )
        .unwrap();
        println!("{:?}", doc2);*/
    }
}
