#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortRange {
    pub start: u16,
    pub end: Option<u16>,
}

impl PortRange {
    pub const fn new(start: u16, end: Option<u16>) -> Self {
        Self { start, end }
    }

    /// A range that matches a single port.
    pub const fn single(port: u16) -> Self {
        Self {
            start: port,
            end: None,
        }
    }

    /// An inclusive range of ports.
    pub const fn range(start: u16, end: u16) -> Self {
        Self {
            start,
            end: Some(end),
        }
    }

    pub fn contains(&self, port: u16) -> bool {
        match self.end {
            Some(end) => port >= self.start && port <= end,
            None => port == self.start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_single_and_ranges() {
        assert!(PortRange::single(25565).contains(25565));
        assert!(!PortRange::single(25565).contains(25566));

        let range = PortRange::range(27015, 27030);

        assert!(range.contains(27015));
        assert!(range.contains(27030));
        assert!(!range.contains(27031));
    }
}
