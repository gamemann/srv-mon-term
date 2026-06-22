use crate::query::types::a2s::QueryA2sCtx;

pub mod a2s;

pub enum Query {
    A2s(QueryA2sCtx),
}
