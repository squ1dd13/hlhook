//! High-level cross-platform hooking library built on top of
//! [`dobby-rs`](https://github.com/black-binary/dobby-rs).
//!
//! This crate aims to make hooking simpler by providing macros to work around type difficulties
//! that are inherent when hooking arbitrary functions. Macros are also provided to reduce the
//! boilerplate associated with retaining trampoline function pointers.
//!
//! **This entire crate is `unsafe`, so be careful!**
//!
//! # Example
//! Here we hook `target` and add `1` to the return value.
//!
//! ```rust
//! make_trampoline!(unsafe fn() -> u8, ORIGINAL);
//!
//! fn target() -> u8 {
//!     10
//! }
//!
//! // This could also be a normal `fn`.
//! let add_one = || {
//!     // Note that we call the result of `get_trampoline`.
//!     let orig_val = unsafe { get_trampoline!(ORIGINAL)() };
//!
//!     orig_val + 1
//! };
//!
//! unsafe { hook!(fn() -> u8, ORIGINAL, target, add_one) }.expect("hook failed");
//!
//! // ...
//!
//! let value = target();
//! assert_eq!(value, 11);
//! ```

use dobby_rs::DobbyHookError;

/// Modifies `target`'s implementation such that it redirects to `replacement`. On success, this
/// function will return `Ok` with a trampoline function pointer that can be used from anywhere to
/// call the original implementation of `target` (bypassing the redirection to `replacement`).
///
/// # Safety
/// Hooking is inherently unsafe. It is up to the caller to ensure that the signatures of `target`
/// and `replacement` are truly compatible.
pub unsafe fn install_hook<F: Copy>(target: F, replacement: F) -> Result<F, DobbyHookError> {
    // Our function only enforces that the target and replacement have the same type for safety,
    // but when actually hooking we erase the type information and use raw addresses.
    let target_addr: dobby_rs::Address = std::mem::transmute_copy(&target);
    let replacement_addr: dobby_rs::Address = std::mem::transmute_copy(&replacement);

    let trampoline_addr = dobby_rs::hook(target_addr, replacement_addr)?;

    // Add back pseudo-type-safety by returning a function pointer matching `target` and
    // `replacement` instead of the raw trampoline address.
    Ok(std::mem::transmute_copy(&trampoline_addr))
}

/// Wraps `install_hook`, casting both function pointers to the same type. If a trampoline variable
/// is provided, it will be set to the trampoline function pointer after hooking.
///
/// # Example
/// Here we hook `target` and add `1` to the return value.
///
/// ```rust
/// make_trampoline!(unsafe fn() -> u8, ORIGINAL);
///
/// fn target() -> u8 {
///     10
/// }
///
/// // This could also be a normal `fn`.
/// let add_one = || {
///     // Note that we call the result of `get_trampoline`.
///     let orig_val = unsafe { get_trampoline!(ORIGINAL)() };
///
///     orig_val + 1
/// };
///
/// unsafe { hook!(fn() -> u8, ORIGINAL, target, add_one) }.expect("hook failed");
///
/// // ...
///
/// let value = target();
/// assert_eq!(value, 11);
/// ```
#[macro_export]
macro_rules! hook {
    ($t:ty, $target:expr, $replacement:expr) => {
        install_hook($target as $t, $replacement as $t)
    };

    ($t:ty, $trampoline:ident, $target:expr, $replacement:expr) => {
        install_hook($target as $t, $replacement as $t).map(|t_ptr| {
            $trampoline = Some(t_ptr);
        })
    };
}

/// Declares a static variable that can be used to store a trampoline function pointer. Typically
/// used in conjunction with [`hook`].
#[macro_export]
macro_rules! make_trampoline {
    ($t:ty, $name:ident) => {
        static mut $name: Option<$t> = None;
    };
}

/// Attempts to get the function pointer from a trampoline variable. Panics if the value is not
/// set.
#[macro_export]
macro_rules! get_trampoline {
    ($name:ident) => {
        $name.expect("trampoline not set")
    };
}
