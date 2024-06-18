#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(doc, feature = "std")), no_std)]
#![deny(dead_code)] // Don't allow missing implementations

/// Abort the process, as if calling [`std::process::abort`]
/// or the C standard library [`abort`](https://en.cppreference.com/w/c/program/abort) function.
///
/// This immediately terminates the process,
/// without calling any destructors or exit codes.
///
///
/// ## Implementations
/// The preferred implementations delegate to a platform-specific abort function.
/// This is enabled whenever `feature = "std"` or `feature = "libc"` is enabled.
/// Using a preferred implementation is equivalent to calling [`immediate_abort`].
///
/// When a platform-specific abort function is not available,
/// this will fall back to using a `panic!` as described in the below section.
///
/// ## Safety
/// This function is **guaranteed** to terminate the process.
/// Unlike the `panic!` function,
/// this function will never unwind into caller code.
///
/// After aborting, this process will never execute any further user code.
/// It is possible some `panic!` code will run inside this function.
/// See the section below for more details.
///
/// ### Invoking `panic!` as fallback
/// No user code will run after invoking this function.
/// However, one of the fallback implementations uses [`core::panic!`] internally.
/// This will trigger the panic hook and run code from the standard library.
/// Outside a call to an `abort` function,
/// this is guaranteed to be the only other code invoked.
/// It will always be passed a `&'static str` argument,
/// which reduces or eliminates use of `core::fmt` machinery.
///
/// For safe code this shouldn't be much of an issue
/// unless you are bothered by the panic hook printing to standard error.
/// To avoid printing to standard output,
/// the easiest workaround is to enable an alternate implementation (see below).
///
/// For unsafe code, there may be further problems.
/// If `unsafe` invariants have been violated,
/// it may be unsafe to execute any code whatever
/// and the abort must be immediate.
///
/// If this usage is unacceptable, invoke the [`immediate_abort`] function instead.
/// This function will never use the fallback implementation.
/// If the primary implementation is missing,
/// it will simply be missing from the library (removed with `cfg`).
/// Alternatively, using the stdlib "panic_immediate_abort" feature should have a similar effect
/// and using the fallback implementation will be fine.
/// As a third choice, `feature = "always-immediate-abort"` will trigger a global compilation error
/// rather than use the fallback implementation.
#[cold]
#[inline(always)] // immediately delegates
pub fn abort() -> ! {
    #[cfg(not(abort_impl = "fallback"))]
    {
        immediate_abort()
    }
    // fallback
    #[cfg(abort_impl = "fallback")]
    {
        fallback_abort()
    }
}

/// Immediately call the platform-specific [`abort`] implementation,
/// without invoking any other code.
///
/// Unlike ,
/// this will never use a fallback implementation that calls `panic!`.
/// Instead, this function will simply not exist.
///
/// In most cases (especially safe code),
/// using the regular [`abort`] function is fine.
#[cfg(not(abort_impl = "fallback"))]
#[inline(always)] // immediately delegates
pub fn immediate_abort() -> ! {
    // implicitly requires std
    #[cfg(abort_impl = "std")]
    {
        std::process::abort();
    }
    // use standard C library abort function
    #[cfg(abort_impl = "libc")]
    unsafe {
        libc::abort();
    }
}

/// The fallback implementation
///
/// ## Rationale for never inlining
/// The most important reason this function should never be inlined
/// is because calling `panic!` might trigger unwinding.
/// We want to guarantee this never happens.
///
/// The secondary reason is that inlining this code would bloat the caller
/// and aborts should always be on the cold-path.
/// The double-panic implementation is two direct calls instead of one.
/// Even the `panic="abort"` case is not inlined,
/// because calling a single-argument function
/// requires an additional load & register move
/// than calling a zero-argument function.
#[cfg(abort_impl = "fallback")]
#[inline(never)]
#[cold]
fn fallback_abort() -> ! {
    #[cfg(feature = "always-immediate-abort")]
    {
        compile_error!("Missing `immediate_abort()` implementation but fallback disabled.")
    }
    #[inline(always)]
    fn do_panic() -> ! {
        panic!("fatal error - aborting");
    }
    /*
     * Check if a panics cause unwinding or immediate aborts.
     * If it aborts, we only need to panic once.
     * If it unwinds, we need to do a double-panic.
     *
     * NOTE: cfg!(panic = "abort") was stabilized in rust 1.60.0.
     * While unknown cfg!(...) attributes would normally evaluate to false,
     * for a couple of versions even mentioning this attribute required
     * a nightly compiler.
     * To avoid errors on old stable compilers,
     * we gate on the compiler version with #[rustversion::since(...))]
     */
    const PANIC_DOES_ABORT: bool = {
        #[cfg(has_cfg_panic)]
        {
            cfg!(panic = "abort")
        }
        #[cfg(not(has_cfg_panic))]
        {
            false
        }
    };
    if PANIC_DOES_ABORT {
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
