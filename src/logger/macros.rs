#[macro_export]
macro_rules! log {
    ($logger:expr, $level:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg($level, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg(LogLevel::Fatal, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg(LogLevel::Error, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg(LogLevel::Warn, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg(LogLevel::Info, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg(LogLevel::Debug, msg).await.ok();
    };
}

#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $logger.log_msg(LogLevel::Trace, msg).await.ok();
    };
}
