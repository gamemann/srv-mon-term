use anyhow::{Result, anyhow};

use crate::server::types::Server;

impl Server {
    pub fn new(ip: String, port: u16, port_query: Option<u16>) -> Self {
        Self {
            ip,
            port,
            port_query,
            ..Default::default()
        }
    }

    pub fn from_addr(addr: &str) -> Result<Self> {
        let mut parts = addr.split(':');

        let ip = parts
            .next()
            .ok_or_else(|| anyhow!("Malformed address"))?
            .to_string();

        let port_str = parts
            .next()
            .ok_or_else(|| anyhow!("Malformed address: missing port"))?;

        let port = port_str
            .parse()
            .map_err(|_| anyhow!("Malformed address: invalid port"))?;

        Ok(Self::new(ip, port, None))
    }
}
