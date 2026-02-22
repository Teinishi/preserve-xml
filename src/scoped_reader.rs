use super::{ParseContent, WithMetadata};

use quick_xml::{Reader, events::Event};
use std::io::BufRead;

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

    pub fn parse_content<T, C: ParseContent<T>, F>(
        &mut self,
        content: &mut C,
        mut f: F,
    ) -> quick_xml::Result<()>
    where
        F: FnMut(&mut ScopedReader<'a, R>, Event) -> quick_xml::Result<Option<WithMetadata<T>>>,
        R: BufRead,
    {
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
            if let Some(data) = f(self, event)? {
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
        Ok(())
    }
}
