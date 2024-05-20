#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(doc, feature = "std")), no_std)]

use cfg_if::cfg_if;

#[inline(always)]
fn _do_panic() -> ! {
    panic!("fatal error - aborting");
}

/// Abort the process, as if calling [`std::process::abort`]
/// or the C standard library [`abort`](https://en.cppreference.com/w/c/program/abort) function.
///
/// This immediately terminates the process,
/// without calling any destructors or exit codes.
///
/// ## Safety
/// This function is **guarenteed** to terminate the process.
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
    cfg_if! {
        // implicitly requries std
        if #[cfg(feature = "std")] {
            std::process::abort();
        } else if #[cfg(feature = "libc")] {
            libc::abort();
        } else if #[cfg(panic = "abort")] {
            /*
             * NOTE: cfg(panic = "abort") only exists on rustc >= 1.60.0
             * On previous versions, it will implicitly evaluate to false.
             */
            #[not(cfg(panic = "abort"))] {
                compile_error!("panicking should abort instead of unwinding")
            }
            _do_panic()
        } else {
            // double panics are guarenteed to abort
            struct DoublePanicGuard;
            impl Drop for DoublePanicGuard {
                fn drop(&mut self) {
                    _do_panic(); // this will abort the process
                }
            }
            let _guard = DoublePanicGuard;
            _do_panic()
        }
    }
}

/// A RAII guard that [aborts](`abort`) the process unless it is explicitly [defused](`AbortGuard::defuse`).
///
/// This is very useful for guarenteeing a section of code will never panic,
/// trivially ensuring the [exception
/// safety](https://doc.rust-lang.org/nomicon/exception-safety.html) of unsafe code.
#[derive(Debug)]
pub struct AbortGuard {
    _priv: (),
}
impl AbortGuard {
    /// Defuse the guard, preventing the drop function from calling [`abort`].
    ///
    /// This is typically used after succesfull execution of some code.
    #[inline]
    pub fn defuse(self) {
        core::mem::forget(self)
    }
}
impl Drop for AbortGuard {
    #[cold]
    #[inline]
    fn drop(&mut self) {
        crate::abort();
    }
}
