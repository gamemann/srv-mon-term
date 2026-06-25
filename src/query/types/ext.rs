use crate::server::{
    data::{ServerOs, ServerStatus},
    user::ServerUser,
    var::ServerVar,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryResponse<T> {
    pub status: ServerStatus,
    pub status_code: Option<u16>,

    pub latency: u64,
    pub data: T,
}

impl<T> Default for QueryResponse<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            status: ServerStatus::Offline,
            status_code: None,
            latency: 0,
            data: T::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct InfoResponse {
    pub srv_name: Option<String>,
    pub map_name: Option<String>,
    pub game_name: Option<String>,

    pub game_dir: Option<String>,
    pub game_id: Option<u16>,

    pub users_cnt: u16,
    pub users_max: u16,
    pub bots_cnt: Option<u16>,

    pub os: Option<ServerOs>,

    pub is_secure: bool,
    pub is_dedicated: bool,

    pub is_public: bool,

    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UsersResponse {
    pub users: Vec<ServerUser>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct VarsResponse {
    pub vars: Vec<ServerVar>,
}
