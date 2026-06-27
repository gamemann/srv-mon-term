use a2s::A2SClient;
use anyhow::{Result, bail};

use crate::{
    query::{
        port::PortRange,
        types::{Query, a2s::QueryA2sCtx},
    },
    server::types::query::ServerQueryType,
};

impl Query {
    pub async fn from_srv_type(query_type: &ServerQueryType) -> Result<Self> {
        match query_type {
            ServerQueryType::A2s => {
                let cl = match A2SClient::new().await {
                    Ok(cl) => cl,
                    Err(e) => bail!("Failed to create A2S client: {e}"),
                };

                Ok(Query::A2s(QueryA2sCtx::new(cl)))
            }
        }
    }

    pub fn get_query_type_from_port(port: u16) -> Option<ServerQueryType> {
        if (PortRange {
            start: 27015,
            end: Some(27030),
        }
        .contains(port))
        {
            return Some(ServerQueryType::A2s);
        }

        None
    }
}
