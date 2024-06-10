#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(doc, feature = "std")), no_std)]

/// Abort the process, as if calling [`std::process::abort`]
/// or the C standard library [`abort`](https://en.cppreference.com/w/c/program/abort) function.
///
/// This immediately terminates the process,
/// without calling any destructors or exit codes.
///
/// ## Safety
/// This function is **guaranteed** to terminate the process.
/// Unlike the `panic!` function,
/// this function will never unwind.
///
/// After aborting, no further code will be ever be executed by this process.
/// This can be used to once critical ,
/// without
#[cold]
#[cfg_attr(not(any(feature = "std", feature = "libc")), track_caller)]
#[cfg_attr(any(feature = "std", feature = "libc", panic = "abort"), inline)]
pub fn abort() -> ! {
    // implicitly requires std
    #[cfg(feature = "std")]
    {
        std::process::abort();
    }
    // use standard C library abort function
    #[cfg(all(feature = "libc", not(feature = "std")))]
    unsafe {
        libc::abort();
    }
    // fallback
    #[cfg(not(any(feature = "std", feature = "libc")))]
    {
        #[inline(always)]
        fn do_panic() -> ! {
            panic!("fatal error - aborting");
        }
        /*
         * Check if a panics cause unwinding or immediate aborts.
         * If it aborts, we only need to panic once.
         * If it unwinds, we need to do a double-panic.
         *
         * NOTE: cfg(panic = "abort") only exists on rustc >= 1.60.0
         * On previous versions, it will implicitly evaluate to false.
         */
        if cfg!(panic = "abort") {
            do_panic()
        } else {
            // double panics are guarenteed to abort
            struct DoublePanicGuard;
            impl Drop for DoublePanicGuard {
                #[inline]
                fn drop(&mut self) {
                    do_panic(); // this will abort the process
                }
            }
            let _guard = DoublePanicGuard;
            do_panic()
        }
    }
}

/// A RAII guard that [aborts](`abort`) the process unless it is explicitly [defused](`AbortGuard::defuse`).
///
/// This is very useful for guarenteeing a section of code will never panic,
/// trivially ensuring the [exception
/// safety](https://doc.rust-lang.org/nomicon/exception-safety.html) of unsafe code.
#[derive(Debug, Clone)]
pub struct AbortGuard {
    _priv: (),
}
impl AbortGuard {
    /// Create a new abort guard,
    /// which will trigger an abort unless [`defuse`](Self::defuse) is called.
    #[inline]
    pub fn new() -> Self {
        AbortGuard { _priv: () }
    }

    /// Defuse the guard, preventing the drop function from calling [`abort`].
    ///
    /// This is typically used after succesfull execution of some code.
    #[inline]
    pub fn defuse(self) {
        core::mem::forget(self)
    }
}
impl Default for AbortGuard {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for AbortGuard {
    #[cold]
    #[inline]
    fn drop(&mut self) {
        crate::abort();
    }
}
