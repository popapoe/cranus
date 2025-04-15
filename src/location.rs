#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

impl Location {
    pub fn from_indexs(line: u32, column: u32) -> Self {
        Location { line, column }
    }
    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }
    pub fn next_column(&mut self) {
        self.column += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)?;
        Ok(())
    }
}
