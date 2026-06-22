use std::sync::Arc;

use tokio::sync::Mutex;

use anyhow::{Result, anyhow};

use crate::context::{Context, ContextWeak};

pub struct StoreCtx<T> {
    pub ctx: Option<ContextWeak>,
    pub store: Arc<Mutex<T>>,
}

impl<T> StoreCtx<T> {
    pub fn new(store: T) -> Self {
        StoreCtx {
            ctx: None,
            store: Arc::new(Mutex::new(store)),
        }
    }

    pub fn ctx(&self) -> Result<Context> {
        self.ctx
            .as_ref()
            .ok_or_else(|| anyhow!("Context not set for store"))?
            .upgrade()
            .ok_or_else(|| anyhow!("Context has been dropped for store"))
    }
}
