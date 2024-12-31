use std::ops::Range;

/// First 24 bits: byte start
///
/// Next 14 bits: bytes count
///
/// Next 16 bits: line
///
/// Last 10 bits: line count
#[derive(Debug, Clone, Copy)]
pub struct Span(u64);

impl Span {
    pub fn new(byte_start: usize, byte_count: usize, line: usize, line_count: usize) -> Self {
        #[cfg(debug_assertions)]
        {
            assert!(byte_start <= (const { (2_usize).pow(24) }));
            assert!(byte_count <= (const { (2_usize).pow(14) }));
            assert!(line <= (const { (2_usize).pow(16) }));
            assert!(line_count <= (const { (2_usize).pow(10) }));
        }

        let (byte_start, byte_count, line, line_count) = (
            byte_start as u64,
            byte_count as u64,
            line as u64,
            line_count as u64,
        );

        Self((byte_start << 40) | (byte_count << 26) | (line << 10) | line_count)
    }

    pub fn merge(span1: Span, span2: Span) -> Span {
        let byte_start = span1.get_byte_start().min(span2.get_byte_start());
        let byte_count = span1.get_byte_end().max(span2.get_byte_end()) - byte_start;

        let line = span1.get_line().min(span2.get_line());
        let line_count = span1.get_line_end().max(span2.get_line_end()) - line;

        Self::new(byte_start, byte_count, line, line_count)
    }

    pub fn dummy() -> Self {
        Self(0)
    }

    pub fn get_byte_start(&self) -> usize {
        (self.0 >> 40) as usize
    }

    pub fn get_byte_count(&self) -> usize {
        ((self.0 >> 26) & 0x3fff) as usize
    }

    pub fn get_byte_end(&self) -> usize {
        self.get_byte_start() + self.get_byte_count()
    }

    pub fn get_byte_range(&self) -> Range<usize> {
        self.get_byte_start()..self.get_byte_end()
    }

    pub fn get_len(&self) -> usize {
        self.get_byte_end() - self.get_byte_start()
    }

    pub fn get_line(&self) -> usize {
        ((self.0 >> 10) & 0xffff) as usize
    }

    pub fn get_line_count(&self) -> usize {
        (self.0 & 0x3ff) as usize
    }

    pub fn get_line_end(&self) -> usize {
        self.get_line() + self.get_line_count()
    }
}

#[cfg(test)]
mod test {
    use crate::Span;

    #[test]
    fn create_span() {
        let byte_start = 2;
        let byte_count = 3;
        let line = 2;
        let line_count = 3;
        let span = Span::new(byte_start, byte_count, line, line_count);
        assert_eq!(byte_start, span.get_byte_start());
        assert_eq!(byte_count, span.get_byte_count());
        assert_eq!(line, span.get_line());
        assert_eq!(line_count, span.get_line_count());
    }

    #[test]
    fn create_merge_span() {
        let span1 = Span::new(2, 3, 2, 3);
        let span2 = Span::new(3, 4, 3, 5);
        let span = Span::merge(span1, span2);
        assert_eq!(2, span.get_byte_start());
        assert_eq!(5, span.get_byte_count());
        assert_eq!(2, span.get_line());
        assert_eq!(6, span.get_line_count());
    }
}
