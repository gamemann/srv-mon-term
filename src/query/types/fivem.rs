/// FiveM / RedM (CitizenFX) query context, which exposes JSON endpoints over HTTP.
#[derive(Debug, Clone, Default)]
pub struct QueryFiveMCtx {}

impl QueryFiveMCtx {
    pub fn new() -> Self {
        Self::default()
    }
}
