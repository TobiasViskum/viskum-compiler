#[derive(Debug, Clone, Copy)]
pub struct Span {
    byte_start: usize,
    byte_end: usize,
    line: usize,
}

impl Span {
    pub fn new(byte_start: usize, byte_end: usize, line: usize) -> Self {
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
        self.byte_start
    }

    pub fn get_byte_end(&self) -> usize {
        self.byte_end
    }

    pub fn get_len(&self) -> usize {
        self.byte_end - self.byte_start
    }
}
