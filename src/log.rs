/// Contains macros for logging. The macros are essentially wrappers around
/// the println macro.
#[macro_export]
macro_rules! log {
    ($s:expr) => ({
        print!("\x1b[32;1m[Log] \x1b[0m");
        println!($s)
    });
    ($fmt:expr, $($arg:tt)*) => ({
        print!("\x1b[32;1m[Log] \x1b[0m");
        print!(concat!($fmt, "\n"), $($arg)*);
    });
}

macro_rules! log_file {
    ($s:expr) => ({
        let _ = LOG_FILE.lock().unwrap().write_all(&format!(concat!($s, "\n")).into_bytes());
    });
    ($fmt:expr, $($arg:tt)*) => ({
        let _ = LOG_FILE.lock().unwrap().write_all(&format!(concat!($fmt, "\n"), $($arg)*).into_bytes());
    });
}

#[macro_export]
macro_rules! error {
    ($s:expr) => ({
        print!("\x1b[31;1m[Error] \x1b[0m");
        println!($s);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        print!("\x1b[31;1m[Error] \x1b[0m");
        print!(concat!($fmt, "\n"), $($arg)*);
    })
}
