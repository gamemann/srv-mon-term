use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// The protocol used to query a server.
///
/// The numeric mapping is persisted by the SQLite store, so existing values must keep their
/// number when new protocols are added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServerQueryType {
    #[default]
    A2s,
    Quake3,
    Minecraft,
    Bedrock,
    GameSpy1,
    GameSpy3,
    FiveM,
}

impl ServerQueryType {
    pub const ALL: [ServerQueryType; 7] = [
        ServerQueryType::A2s,
        ServerQueryType::Quake3,
        ServerQueryType::Minecraft,
        ServerQueryType::Bedrock,
        ServerQueryType::GameSpy1,
        ServerQueryType::GameSpy3,
        ServerQueryType::FiveM,
    ];

    /// The primary name accepted on the command line.
    pub fn name(&self) -> &'static str {
        match self {
            ServerQueryType::A2s => "a2s",
            ServerQueryType::Quake3 => "quake3",
            ServerQueryType::Minecraft => "minecraft",
            ServerQueryType::Bedrock => "bedrock",
            ServerQueryType::GameSpy1 => "gamespy1",
            ServerQueryType::GameSpy3 => "gamespy3",
            ServerQueryType::FiveM => "fivem",
        }
    }

    /// Alternative names accepted on the command line.
    pub fn aliases(&self) -> &'static [&'static str] {
        match self {
            ServerQueryType::A2s => &["source", "valve", "steam", "goldsrc"],
            ServerQueryType::Quake3 => &["q3", "quake", "cod", "cod2", "cod4", "codmp", "et"],
            ServerQueryType::Minecraft => &["mc", "minecraft-java", "java", "slp"],
            ServerQueryType::Bedrock => &["mcbe", "mcpe", "minecraft-bedrock", "raknet"],
            ServerQueryType::GameSpy1 => &["gs1", "gamespy"],
            ServerQueryType::GameSpy3 => &["gs3", "minecraft-query", "mcquery", "bf2"],
            ServerQueryType::FiveM => &["cfx", "gta5", "redm", "citizenfx"],
        }
    }

    /// Short human readable summary of what the protocol covers.
    pub fn description(&self) -> &'static str {
        match self {
            ServerQueryType::A2s => "Source/GoldSrc engine servers (CS2, TF2, Garry's Mod, Rust)",
            ServerQueryType::Quake3 => {
                "Quake 3 engine servers (Call of Duty 1/2/4, Quake 3, Wolfenstein: ET)"
            }
            ServerQueryType::Minecraft => "Minecraft: Java Edition (Server List Ping)",
            ServerQueryType::Bedrock => "Minecraft: Bedrock Edition (RakNet unconnected ping)",
            ServerQueryType::GameSpy1 => "GameSpy v1 servers (Unreal Tournament, Battlefield 1942)",
            ServerQueryType::GameSpy3 => {
                "GameSpy v3 servers (Battlefield 2, Minecraft query listener)"
            }
            ServerQueryType::FiveM => "FiveM/RedM (CitizenFX) servers over HTTP",
        }
    }

    /// The port a server of this type usually listens on.
    pub fn default_port(&self) -> u16 {
        match self {
            ServerQueryType::A2s => 27015,
            ServerQueryType::Quake3 => 28960,
            ServerQueryType::Minecraft => 25565,
            ServerQueryType::Bedrock => 19132,
            ServerQueryType::GameSpy1 => 7778,
            ServerQueryType::GameSpy3 => 25565,
            ServerQueryType::FiveM => 30120,
        }
    }
}

impl From<i32> for ServerQueryType {
    fn from(value: i32) -> Self {
        match value {
            0 => ServerQueryType::A2s,
            1 => ServerQueryType::Quake3,
            2 => ServerQueryType::Minecraft,
            3 => ServerQueryType::Bedrock,
            4 => ServerQueryType::GameSpy1,
            5 => ServerQueryType::GameSpy3,
            6 => ServerQueryType::FiveM,
            _ => ServerQueryType::default(),
        }
    }
}

impl From<ServerQueryType> for i32 {
    fn from(value: ServerQueryType) -> Self {
        match value {
            ServerQueryType::A2s => 0,
            ServerQueryType::Quake3 => 1,
            ServerQueryType::Minecraft => 2,
            ServerQueryType::Bedrock => 3,
            ServerQueryType::GameSpy1 => 4,
            ServerQueryType::GameSpy3 => 5,
            ServerQueryType::FiveM => 6,
        }
    }
}

impl FromStr for ServerQueryType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let needle = s.trim().to_lowercase().replace(['_', ' '], "-");

        ServerQueryType::ALL
            .into_iter()
            .find(|t| {
                t.name() == needle
                    || t.aliases().contains(&needle.as_str())
                    || t.to_string().to_lowercase() == needle
            })
            .ok_or(())
    }
}

impl fmt::Display for ServerQueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ServerQueryType::A2s => "A2S",
            ServerQueryType::Quake3 => "Quake3",
            ServerQueryType::Minecraft => "Minecraft",
            ServerQueryType::Bedrock => "Bedrock",
            ServerQueryType::GameSpy1 => "GameSpy1",
            ServerQueryType::GameSpy3 => "GameSpy3",
            ServerQueryType::FiveM => "FiveM",
        };

        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_aliases() {
        assert_eq!("a2s".parse(), Ok(ServerQueryType::A2s));
        assert_eq!("SOURCE".parse(), Ok(ServerQueryType::A2s));
        assert_eq!("cod4".parse(), Ok(ServerQueryType::Quake3));
        assert_eq!("mc".parse(), Ok(ServerQueryType::Minecraft));
        assert_eq!("minecraft_bedrock".parse(), Ok(ServerQueryType::Bedrock));
        assert_eq!("FiveM".parse(), Ok(ServerQueryType::FiveM));
        assert_eq!("nope".parse::<ServerQueryType>(), Err(()));
    }

    #[test]
    fn round_trips_store_ids() {
        for query_type in ServerQueryType::ALL {
            let id: i32 = query_type.into();

            assert_eq!(ServerQueryType::from(id), query_type);
        }
    }

    #[test]
    fn ids_are_stable() {
        // Changing these breaks existing SQLite stores.
        assert_eq!(i32::from(ServerQueryType::A2s), 0);
        assert_eq!(i32::from(ServerQueryType::Quake3), 1);
    }
}
