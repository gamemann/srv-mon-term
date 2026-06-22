use std::{sync::OnceLock, time::Instant};

static START: OnceLock<Instant> = OnceLock::new();

pub fn get_now_ms() -> u128 {
    START.get_or_init(Instant::now).elapsed().as_millis()
}
