use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    byte_start: u32,
    byte_end: u32,
    line: u32,
    /*
    start_line: u16,
    end_line: u16,
    */
}

impl Span {
    pub fn new(byte_start: usize, byte_end: usize, line: usize) -> Self {
        Self { byte_start: byte_start as u32, byte_end: byte_end as u32, line: line as u32 }
    }

    pub fn merge(span1: Span, span2: Span) -> Span {
        let byte_start = span1.byte_start.min(span2.byte_start);
        let byte_end = span1.byte_end.max(span2.byte_end);
        let line = span1.line.min(span2.line);
        Self { byte_start, byte_end, line }
    }

    pub fn dummy() -> Self {
        Self {
            byte_start: 0,
            byte_end: 0,
            line: 1,
        }
    }

    pub fn get_byte_start(&self) -> usize {
        self.byte_start as usize
    }

    pub fn get_byte_end(&self) -> usize {
        self.byte_end as usize
    }

    pub fn get_byte_range(&self) -> Range<usize> {
        self.get_byte_start()..self.get_byte_end()
    }

    pub fn get_len(&self) -> usize {
        self.get_byte_end() - self.get_byte_start()
    }

    pub fn get_line(&self) -> usize {
        self.line as usize
    }
}

#[cfg(test)]
mod test {
    use crate::Span;

    #[test]
    fn create_span() {
        let byte_start = 2;
        let byte_end = 3;
        let line = 2;
        let span = Span::new(byte_start, byte_end, line);
        assert_eq!(byte_start, span.get_byte_start());
        assert_eq!(byte_end, span.get_byte_end());
        assert_eq!(line, span.get_line())
    }
}
