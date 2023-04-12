# `scratchbuffer`

[![Crates.io](https://img.shields.io/crates/v/scratchbuffer.svg?label=scratchbuffer)](https://crates.io/crates/scratchbuffer)
[![docs.rs](https://docs.rs/scratchbuffer/badge.svg)](https://docs.rs/scratchbuffer/)
[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust CI](https://github.com/HellButcher/scratchbuffer-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/HellButcher/scratchbuffer-rs/actions/workflows/rust.yml)

<!-- Short Introduction -->

`ScratchBuffer<dyn Trait>` is like `Vec<Box<dyn Trait>>`, but with an optimization that avoids reallocations. It allows to store multiple items of the same type in a continous chunk of memory, and get a slice. When the ScratchBuffer is cleared, it can be re-used for multiple items of a different type. It doesn't re-allocate memory when not needed, even when re-used as a different type.

## Example

This example stores multiple values the `ScratchBuffer` and accesses them.
(`push(...)` requires the `"unstable"` feature (**nightly only**))

```rust
use scratchbuffer::ScratchBuffer;
let mut buf = ScratchBuffer::new();

// to use the scratchbuffer, you need to clear it, and tell which type you want to use
let mut u32buf = buf.clear_and_use_as::<u32>();
u32buf.push(123);
u32buf.push(456);
assert_eq!(&[123u32, 456], u32buf.as_slice());

// now use the scratchbuffer with a different type.
// in this case, no memory-allocations are needed, because the scratchbuffer
// can re-use the memory it has allocated for u32buf
let u16buf = buf.clear_and_use_as::<u16>();
u16buf.push(345);
u16buf.push(678);
assert_eq!(&[345u16, 678], u16buf.as_slice());
```

## `no_std`

This crate should also work without `std` (with `alloc`). No additional configuration required.

## License

[license]: #license

This project is licensed under either of

- MIT license ([LICENSE-MIT] or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0, ([LICENSE-APACHE] or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[license-mit]: ./LICENSE-MIT
[license-apache]: ./LICENSE-APACHE
