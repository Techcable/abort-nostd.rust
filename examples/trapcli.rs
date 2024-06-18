//! A simple command-line app that immediately traps.
//!
//! Prints a `"Calling libabort::trap()"` message to stderr first,
//! unless passed environment variable `SILENT` (env. var ignored)
//!
//! See `abortcli` for the counterpart that calls `abort()`.

fn main() {
    if std::env::var_os("SILENT").is_none() {
        eprintln!("Calling libabort::trap()");
    }
    libabort::trap();
}
