/// Minecraft: Java Edition query context (Server List Ping over TCP).
#[derive(Debug, Clone, Default)]
pub struct QueryMinecraftCtx {}

impl QueryMinecraftCtx {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Minecraft: Bedrock Edition query context (RakNet unconnected ping over UDP).
#[derive(Debug, Clone, Default)]
pub struct QueryBedrockCtx {}

impl QueryBedrockCtx {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Fields carried by a Bedrock unconnected pong MOTD string.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BedrockStatus {
    pub edition: Option<String>,
    pub motd: Option<String>,
    pub protocol: Option<String>,
    pub version: Option<String>,
    pub users_cnt: u16,
    pub users_max: u16,
    pub server_id: Option<String>,
    pub level_name: Option<String>,
    pub gamemode: Option<String>,
    pub port_v4: Option<u16>,
}
