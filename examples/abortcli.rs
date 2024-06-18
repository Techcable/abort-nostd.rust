//! A simple command-line app that immediately aborts.
//!
//! Prints a `"Calling libabort::abort()"` message to stderr first,
//! unless passed environment variable `SILENT` (env. var ignored)
//!
//! See `trapcli` for the counterpart that calls `trap()`.

fn main() {
    if std::env::var_os("SILENT").is_none() {
        eprintln!("Calling libabort::abort()");
    }
    libabort::abort();
}
