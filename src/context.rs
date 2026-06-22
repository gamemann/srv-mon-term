use std::sync::{Arc, Weak};

use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tokio_util::sync::CancellationToken;

use crate::{
    cli::args::Args,
    logger::types::Logger,
    server::{context::ServerCtx, types::Server},
    settings::Settings,
    store::types::Store,
    tui::types::Tui,
};

pub struct ContextInner {
    pub args: Args,
    pub logger: RwLock<Logger>,
    pub tui: RwLock<Tui>,
    pub store: RwLock<Store>,

    pub settings: RwLock<Settings>,

    pub servers: RwLock<Vec<Arc<ServerCtx>>>,
    pub sch: RwLock<JobScheduler>,

    pub cancel_token: CancellationToken,
}

pub type Context = Arc<ContextInner>;
pub type ContextWeak = Weak<ContextInner>;

impl ContextInner {
    pub fn new(args: Args, logger: Logger, tui: Tui, store: Store, sch: JobScheduler) -> Context {
        Arc::new(ContextInner {
            args,
            logger: RwLock::new(logger),
            tui: RwLock::new(tui),
            store: RwLock::new(store),
            settings: RwLock::new(Settings::default()),
            servers: RwLock::new(Vec::new()),
            sch: RwLock::new(sch),
            cancel_token: CancellationToken::new(),
        })
    }

    pub async fn get_server_ctx(&self, server: &Server) -> Option<Arc<ServerCtx>> {
        let servers = self.servers.read().await;

        for server_ctx in servers.iter() {
            let server_ctx_server = server_ctx.server.read().await;

            if server_ctx_server.ip == server.ip && server_ctx_server.port == server.port {
                return Some(server_ctx.clone());
            }
        }

        None
    }
}
