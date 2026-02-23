use crate::unescape_xml;

use super::utils::{debug_utf8, escape_xml};

use std::{borrow::Cow, fmt::Debug, io::Write};

#[derive(Clone, PartialEq, Eq)]
pub struct Attributes {
    pub slots: Vec<AttrSlot>,
    pub trailing_space: Vec<u8>,
}

impl Debug for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Attributes {{ slots: {:?}, trailing_space: {:?} }}",
            self.slots,
            debug_utf8(&self.trailing_space)
        )
    }
}

impl Attributes {
    pub fn new<F>(input: &[u8], f: F) -> Self
    where
        F: FnMut(&[u8], String) -> bool,
    {
        let (slots, trailing_space) = AttrScanner::new(input).scan(f);
        Self {
            slots,
            trailing_space,
        }
    }

    pub fn write<'a, W: Write, F>(&self, writer: &mut W, mut f: F) -> std::io::Result<()>
    where
        F: FnMut(&[u8]) -> Option<Cow<'a, str>>,
    {
        for attr in &self.slots {
            match attr {
                AttrSlot::Defined { prefix, key, .. } => {
                    if let Some(value) = f(key) {
                        let (escaped_value, quote) = escape_xml(&value);
                        writer.write_all(prefix)?;
                        write!(writer, "{}{}", escaped_value, quote)?;
                    }
                }
                AttrSlot::Raw(data) => {
                    writer.write_all(data)?;
                }
            }
        }
        writer.write_all(&self.trailing_space)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum AttrSlot {
    Defined {
        prefix: Vec<u8>,
        key: Vec<u8>,
        quote: u8,
    },
    Raw(Vec<u8>),
}

impl Debug for AttrSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined { prefix, key, quote } => write!(
                f,
                "Defined {{ prefix: {:?}, key: {:?}, quote: {:?}}}",
                debug_utf8(prefix),
                debug_utf8(key),
                quote,
            ),
            Self::Raw(data) => {
                write!(f, "Raw({:?})", debug_utf8(data))
            }
        }
    }
}

#[derive(Debug)]
struct AttrScanner<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> AttrScanner<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).cloned()
    }

    fn consume(&mut self) -> Option<u8> {
        let b = self.peek();
        if b.is_some() {
            self.pos += 1;
        }
        b
    }

    fn consume_whitespace(&mut self) -> Vec<u8> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_vec()
    }

    fn scan<F>(&mut self, mut f: F) -> (Vec<AttrSlot>, Vec<u8>)
    where
        F: FnMut(&[u8], String) -> bool,
    {
        let mut slots = Vec::new();
        let trailing_space;

        loop {
            let start_pos = self.pos;
            let leading = self.consume_whitespace();

            // 属性名が始まらないまま末尾に達したなら、それは末尾の空白
            let name_start = self.pos;
            if self.peek().is_none() {
                trailing_space = leading;
                break;
            }

            // 属性名を取得
            while let Some(b) = self.peek() {
                if b == b'=' || b.is_ascii_whitespace() {
                    break;
                }
                self.consume();
            }
            let key = self.input[name_start..self.pos].to_vec();

            self.consume_whitespace();
            if self.consume() != Some(b'=') {
                // 文法エラー
                todo!();
            }
            self.consume_whitespace();

            let quote = self.consume().unwrap_or(b'"');

            // 属性値のパース
            let value_start = self.pos;
            while self.consume().is_some_and(|b| b != quote) {}
            let value_end_with_quote = self.pos;

            // コールバックを呼ぶ
            if f(
                key.as_slice(),
                unescape_xml(&self.input[value_start..value_end_with_quote - 1]),
            ) {
                let prefix = self.input[start_pos..value_start].to_vec();
                slots.push(AttrSlot::Defined { prefix, key, quote });
            } else {
                slots.push(AttrSlot::Raw(
                    self.input[start_pos..value_end_with_quote].to_vec(),
                ));
            }
        }

        (slots, trailing_space)
    }
}
