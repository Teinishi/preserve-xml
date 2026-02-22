use super::{AttrSlot, ParseContent, WithMetadata};

use quick_xml::{Reader, events::Event};
use std::io::BufRead;

#[derive(Debug)]
pub struct ElementBuilder<'a, R> {
    reader: &'a mut Reader<R>,
    stack: Vec<Vec<u8>>,
}

impl<'a, R> ElementBuilder<'a, R> {
    pub fn new(reader: &'a mut Reader<R>) -> Self {
        Self {
            reader,
            stack: Vec::new(),
        }
    }

    fn read<'b>(&mut self, buf: &'b mut Vec<u8>) -> quick_xml::Result<Event<'b>>
    where
        R: BufRead,
    {
        // イベントを1つ読んでスタックに反映する
        let event = self.reader.read_event_into(buf)?;
        match &event {
            Event::Start(e) => {
                self.stack.push(e.name().as_ref().to_vec());
            }
            Event::End(e) => {
                if let Some(s) = self.stack.last()
                    && s == e.name().as_ref()
                {
                    self.stack.pop();
                }
            }
            _ => {}
        }

        Ok(event)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> quick_xml::Result<()>
    where
        R: BufRead,
    {
        // 現在のタグが終わるか、Eof まで進める
        let stack_len = self.stack.len();
        loop {
            let event = self.read(buf)?;
            match &event {
                Event::End(_) if self.stack.len() <= stack_len => {
                    break;
                }
                Event::Eof => {
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub(crate) fn parse_content<T, C: ParseContent<T>, F>(
        &mut self,
        mut read_event: F,
    ) -> quick_xml::Result<C>
    where
        R: BufRead,
        F: FnMut(&mut ElementBuilder<'a, R>, Event) -> quick_xml::Result<Option<WithMetadata<T>>>,
    {
        let mut content = C::new();

        // 子要素を読んでコールバックを呼び、マッチすれば push、しなければ push_raw
        let stack_len = self.stack.len();

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();

        loop {
            let event = self.read(&mut buf2)?;
            match &event {
                Event::End(_) if self.stack.len() <= stack_len => {
                    content.push_raw(buf1);
                    break;
                }
                Event::Eof => {
                    content.push_raw(buf1);
                    break;
                }
                _ => {}
            }

            let is_start = matches!(event, Event::Start(_));

            // コールバックを呼んでマッチしたらその前の生データと子要素を追加
            if let Some(data) = read_event(self, event)? {
                content.push_raw(std::mem::take(&mut buf1));
                content.push(data);
                buf2.clear();
            } else {
                if is_start {
                    // マッチしなかった Start イベントは、対応する End まで進める
                    self.read_to_end(&mut buf2)?;
                }
                buf1.append(&mut buf2);
            }
        }

        Ok(content)
    }

    pub fn build<S, T, C: ParseContent<T>, F1, F2>(
        &mut self,
        attributes: Vec<AttrSlot>,
        read_event: F1,
        finalize: F2,
    ) -> quick_xml::Result<WithMetadata<S>>
    where
        R: BufRead,
        F1: FnMut(&mut ElementBuilder<'a, R>, Event) -> quick_xml::Result<Option<WithMetadata<T>>>,
        F2: FnOnce(C) -> S,
    {
        // todo: 最後まで読んだことを保証する
        let content = self.parse_content(read_event)?;
        let element = finalize(content);
        Ok(WithMetadata::new(attributes, element))
    }
}

// ElementBuilder からしか生成できない構造体で、WithMetadata の生成を制限する
#[derive(Debug)]
#[non_exhaustive]
pub struct EmptyElementBuilder;

impl EmptyElementBuilder {
    pub fn build<T>(&self, attributes: Vec<AttrSlot>, element: T) -> WithMetadata<T> {
        WithMetadata::new(attributes, element)
    }
}

impl<'a, R> From<&mut ElementBuilder<'a, R>> for EmptyElementBuilder {
    fn from(_value: &mut ElementBuilder<'a, R>) -> Self {
        Self
    }
}
