use a2s::A2SClient;
use anyhow::{Result, bail};

use crate::{
    query::types::{
        Query,
        a2s::QueryA2sCtx,
        fivem::QueryFiveMCtx,
        gamespy::{QueryGameSpy1Ctx, QueryGameSpy3Ctx},
        minecraft::{QueryBedrockCtx, QueryMinecraftCtx},
        port::PortRange,
        quake3::QueryQuake3Ctx,
    },
    server::types::query::ServerQueryType,
};

/// Well known port ranges we use to guess a query type when the user didn't specify one.
const PORT_HINTS: &[(PortRange, ServerQueryType)] = &[
    (PortRange::range(27015, 27030), ServerQueryType::A2s),
    (PortRange::single(25565), ServerQueryType::Minecraft),
    (PortRange::range(19132, 19133), ServerQueryType::Bedrock),
    (PortRange::range(28960, 28970), ServerQueryType::Quake3),
    (PortRange::range(27950, 27965), ServerQueryType::Quake3),
    (PortRange::range(30110, 30130), ServerQueryType::FiveM),
    (PortRange::range(23000, 23009), ServerQueryType::GameSpy1),
    (PortRange::single(7778), ServerQueryType::GameSpy1),
];

impl Query {
    pub async fn from_srv_type(query_type: &ServerQueryType) -> Result<Self> {
        Ok(match query_type {
            ServerQueryType::A2s => {
                let cl = match A2SClient::new().await {
                    Ok(cl) => cl,
                    Err(e) => bail!("Failed to create A2S client: {e}"),
                };

                Query::A2s(QueryA2sCtx::new(cl))
            }
            ServerQueryType::Quake3 => Query::Quake3(QueryQuake3Ctx::new()),
            ServerQueryType::Minecraft => Query::Minecraft(QueryMinecraftCtx::new()),
            ServerQueryType::Bedrock => Query::Bedrock(QueryBedrockCtx::new()),
            ServerQueryType::GameSpy1 => Query::GameSpy1(QueryGameSpy1Ctx::new()),
            ServerQueryType::GameSpy3 => Query::GameSpy3(QueryGameSpy3Ctx::new()),
            ServerQueryType::FiveM => Query::FiveM(QueryFiveMCtx::new()),
        })
    }

    /// Guesses the query type from the port a server listens on.
    pub fn get_query_type_from_port(port: u16) -> Option<ServerQueryType> {
        PORT_HINTS
            .iter()
            .find(|(range, _)| range.contains(port))
            .map(|(_, query_type)| *query_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guesses_types_from_ports() {
        assert_eq!(
            Query::get_query_type_from_port(27015),
            Some(ServerQueryType::A2s)
        );
        assert_eq!(
            Query::get_query_type_from_port(25565),
            Some(ServerQueryType::Minecraft)
        );
        assert_eq!(
            Query::get_query_type_from_port(19132),
            Some(ServerQueryType::Bedrock)
        );
        assert_eq!(
            Query::get_query_type_from_port(28960),
            Some(ServerQueryType::Quake3)
        );
        assert_eq!(
            Query::get_query_type_from_port(30120),
            Some(ServerQueryType::FiveM)
        );
        assert_eq!(Query::get_query_type_from_port(1234), None);
    }
}
