pub struct PortRange {
    pub start: u16,
    pub end: Option<u16>,
}

impl PortRange {
    pub fn new(start: u16, end: Option<u16>) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, port: u16) -> bool {
        if let Some(end) = self.end {
            port >= self.start && port <= end
        } else {
            port == self.start
        }
    }
}
