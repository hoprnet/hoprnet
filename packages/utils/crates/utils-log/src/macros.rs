#[macro_export]
macro_rules! log {
    // Just use the "log" crate for now

    // log!(target: "my_target", Level::Info; key1 = 42, key2 = true; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($key:tt = $value:expr),+; $($arg:tt)+) => (
        // ignoring the key-value part, none of the production code uses it at this point
        log::log!(target: $target, lvl: $lvl, $($arg)+)
    );

    // log!(target: "my_target", Level::Info; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        log::log!(target: $target, lvl: $lvl, $($arg)+)
    });


    ($lvl:expr, $($arg:tt)+) => ($crate::downstream_log::log!(target: std::module_path!(), $lvl, $($arg)+));
}

#[macro_export]
macro_rules! trace {
    // trace!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // trace!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::Level::Trace, $($arg)+));

    // trace!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!($crate::Level::Trace, $($arg)+))
}

#[macro_export]
macro_rules! debug {
    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::Level::Debug, $($arg)+));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!($crate::Level::Debug, $($arg)+))
}

#[macro_export]
macro_rules! info {
    // info!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // info!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::Level::Info, $($arg)+));

    // info!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!($crate::Level::Info, $($arg)+))
}

#[macro_export]
macro_rules! warn {
    // warn!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // warn!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::Level::Warn, $($arg)+));

    // warn!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!($crate::Level::Warn, $($arg)+))
}

#[macro_export]
macro_rules! error {
    // error!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // error!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::Level::Error, $($arg)+));

    // error!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!($crate::Level::Error, $($arg)+))
}
