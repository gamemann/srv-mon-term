#[macro_export]
macro_rules! log {
    ($ctx:expr, $level:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);

        Logger::log_msg($ctx.clone(), $level, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($ctx:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        Logger::log_msg($ctx.clone(), LogLevel::Fatal, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_error {
    ($ctx:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        Logger::log_msg($ctx.clone(), LogLevel::Error, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_warn {
    ($ctx:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        Logger::log_msg($ctx.clone(), LogLevel::Warn, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_info {
    ($ctx:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        Logger::log_msg($ctx.clone(), LogLevel::Info, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_debug {
    ($ctx:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        Logger::log_msg($ctx.clone(), LogLevel::Debug, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_trace {
    ($ctx:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        Logger::log_msg($ctx.clone(), LogLevel::Trace, msg).await.ok();
    };
}
