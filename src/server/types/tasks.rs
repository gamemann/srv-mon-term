use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ServerTasks {
    pub query_task_id: Option<u128>,
    pub latency_task_id: Option<u128>,
}
