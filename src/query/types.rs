pub mod a2s;
pub mod ext;
pub mod port;

use crate::query::types::a2s::QueryA2sCtx;

pub enum Query {
    A2s(QueryA2sCtx),
}
