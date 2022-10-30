// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// The standard logging macro.
///
/// This macro will generically log with the specified `Level` and `format!`
/// based argument list.
///
/// # Examples
///
/// ```rust, ignore
/// use ftlog::{log, Level};
///
/// # fn main() {
/// let data = (42, "Forty-two");
/// let private_data = "private";
///
/// log!(Level::Error, "Received errors: {}, {}", data.0, data.1);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! log {

    // log!(target: "my_target", Level::Info; "a {} event", "log");
    (target: $target:expr, $limit:expr, $lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api_log(
                __log_format_args!($($arg)+),
                lvl,
                &($target, __log_module_path!(), __log_file!(), __log_line!(), $limit),
                $crate::__private_api::Option::None,
            );
        }
    });

    // log!(Level::Info, "a log event")
    ($limit:expr, $lvl:expr, $($arg:tt)+) => (log!(target: __log_module_path!(), $limit, $lvl, $($arg)+));
}

/// Logs a message at the error level.
///
/// # Examples
///
/// ```rust
/// use ftlog::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("Error: {} on port {}", err_info, port);
/// error!(limit: 100, "App Error: {}, Port: {}", err_info, 22);//每100ms最多一条日志
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! error {
    (limit: $limit:expr, $($arg:tt)+) => (log!($limit, $crate::Level::Error, $($arg)+));
    // error!("a {} event", "log")
    ($($arg:tt)+) => (log!(0, $crate::Level::Error, $($arg)+))
}

/// Logs a message at the warn level.
///
/// # Examples
///
/// ```rust
/// use ftlog::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("Warning! {}!", warn_description);
/// warn!(limit: 100, "App received warning: {}", warn_description);//每100ms最多一条日志
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! warn {
    (limit: $limit:expr, $($arg:tt)+) => (log!($limit, $crate::Level::Warn, $($arg)+));
    // warn!("a {} event", "log")
    ($($arg:tt)+) => (log!(0, $crate::Level::Warn, $($arg)+))
}

/// Logs a message at the info level.
///
/// # Examples
///
/// ```rust
/// use ftlog::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// info!(limit: 100, "Successfull connection, port: {}, speed: {}",
///       conn_info.port, conn_info.speed);//每100ms最多一条日志
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! info {
    (limit: $limit:expr, $($arg:tt)+) => (log!($limit, $crate::Level::Info, $($arg)+));
    // info!(0, "a {} event", "log")
    ($($arg:tt)+) => (log!(0, $crate::Level::Info, $($arg)+))
}

/// Logs a message at the debug level.
///
/// # Examples
///
/// ```rust
/// use ftlog::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// debug!(limit: 100, "New position: x: {}, y: {}", pos.x, pos.y);//每100ms最多一条日志
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! debug {
    (limit: $limit:expr, $($arg:tt)+) => (log!($limit, $crate::Level::Debug, $($arg)+));
    // debug!("a {} event", "log")
    ($($arg:tt)+) => (log!(0, $crate::Level::Debug, $($arg)+))
}

/// Logs a message at the trace level.
///
/// # Examples
///
/// ```rust
/// use ftlog::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// trace!(limit: 100, "x is {} and y is {}",
///        if pos.x >= 0.0 { "positive" } else { "negative" },
///        if pos.y >= 0.0 { "positive" } else { "negative" });//每100ms最多一条日志
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! trace {
    (limit: $limit:expr, $($arg:tt)+) => (log!($limit, $crate::Level::Trace, $($arg)+));
    // trace!("a {} event", "log")
    ($($arg:tt)+) => (log!(0, $crate::Level::Trace, $($arg)+))
}

/// Determines if a message logged at the specified level in that module will
/// be logged.
///
/// This can be used to avoid expensive computation of log message arguments if
/// the message would be ignored anyway.
///
/// # Examples
///
/// ```rust
/// use ftlog::Level::Debug;
/// use ftlog::{debug, log_enabled};
///
/// # fn foo() {
/// if log_enabled!(Debug) {
///     let data = expensive_call();
///     debug!("expensive debug data: {} {}", data.x, data.y);
/// }
/// # }
/// # struct Data { x: u32, y: u32 }
/// # fn expensive_call() -> Data { Data { x: 0, y: 0 } }
/// # fn main() {}
/// ```
#[macro_export(local_inner_macros)]
macro_rules! log_enabled {
    (target: $target:expr, $lvl:expr) => {{
        let lvl = $lvl;
        lvl <= $crate::STATIC_MAX_LEVEL
            && lvl <= $crate::max_level()
            && $crate::__private_api_enabled(lvl, $target)
    }};
    ($lvl:expr) => {
        log_enabled!(target: __log_module_path!(), $lvl)
    };
}

// The log macro above cannot invoke format_args directly because it uses
// local_inner_macros. A format_args invocation there would resolve to
// $crate::format_args which does not exist. Instead invoke format_args here
// outside of local_inner_macros so that it resolves (probably) to
// core::format_args or std::format_args. Same for the several macros that
// follow.
//
// This is a workaround until we drop support for pre-1.30 compilers. At that
// point we can remove use of local_inner_macros, use $crate:: when invoking
// local macros, and invoke format_args directly.
#[doc(hidden)]
#[macro_export]
macro_rules! __log_format_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_module_path {
    () => {
        module_path!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_file {
    () => {
        file!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_line {
    () => {
        line!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_key {
    // key1 = 42
    ($($args:ident)*) => {
        stringify!($($args)*)
    };
    // "key1" = 42
    ($($args:expr)*) => {
        $($args)*
    };
}
