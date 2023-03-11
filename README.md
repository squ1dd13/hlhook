# hlhook

High-level cross-platform hooking library built on top of
[`dobby-rs`](https://github.com/black-binary/dobby-rs).

This crate aims to make hooking simpler by providing macros to work around type difficulties that
are inherent when hooking arbitrary functions. Macros are also provided to reduce the boilerplate
associated with retaining trampoline function pointers.

**This entire crate is `unsafe`, so be careful!**

## Example

Here we hook `target` and add `1` to the return value.

```rust
make_trampoline!(unsafe fn() -> u8, ORIGINAL);

fn target() -> u8 {
    10
}

// This could also be a normal `fn`.
let add_one = || {
    // Note that we call the result of `get_trampoline`.
    let orig_val = unsafe { get_trampoline!(ORIGINAL)() };

    orig_val + 1
};

unsafe { hook!(fn() -> u8, ORIGINAL, target, add_one) }.expect("hook failed");

// ...

let value = target();
assert_eq!(value, 11);
```
