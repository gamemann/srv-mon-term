use std::sync::Arc;

use tokio::sync::RwLock;

pub struct StoreCtx<T, S, O> {
    pub store: Arc<RwLock<T>>,
    pub state: Arc<RwLock<S>>,
    pub opts: O,
}

impl<T: Default, S: Default, O> StoreCtx<T, S, O> {
    pub fn new(opts: O) -> Self {
        StoreCtx {
            store: Arc::new(RwLock::new(T::default())),
            state: Arc::new(RwLock::new(S::default())),
            opts,
        }
    }
}
