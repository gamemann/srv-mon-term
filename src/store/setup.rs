use anyhow::{Result, bail};

use crate::context::Context;

use crate::store::ext::StoreExt;

pub async fn store_setup(ctx: Context) -> Result<()> {
    let mut store = ctx.store.write().await;

    match store.init().await {
        Ok(_) => (),
        Err(e) => bail!("Failed to initialize store: {}", e),
    }

    Ok(())
}
