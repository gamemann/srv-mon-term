use a2s::A2SClient;

pub enum QueryA2sStatusCodes {
    Ok,
    IoError,
    TryReserveError,
    InvalidResponse,
    MismatchId,
    InvalidBz2Size,
    ChecksumMismatch,
    Other,
}

pub struct QueryA2sCtx {
    pub cl: A2SClient,
}

impl QueryA2sCtx {
    pub fn new(cl: A2SClient) -> Self {
        Self { cl }
    }
}
