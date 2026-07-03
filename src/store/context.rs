use std::sync::Arc;

use tokio::sync::Mutex;

pub struct StoreCtx<T> {
    pub store: Arc<Mutex<T>>,
}

impl<T> StoreCtx<T> {
    pub fn new(store: T) -> Self {
        StoreCtx {
            store: Arc::new(Mutex::new(store)),
        }
    }
}
