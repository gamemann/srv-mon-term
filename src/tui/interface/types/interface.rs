use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum TuiInterfaceType {
    Dashboard,
    Logs,
    Settings,
    About,

    ServerView,
    ServerNew,
    ServerSettings,
}
