//! This library provides slice-like types using a "root + extent" representation
//! internally, which is more friendly to implementing parsing functions in `const`
//! Rust as of version 1.92.0.
//!
//! A standard Rust slice is implemented internally as a "fat pointer", a struct
//! storing two pieces of data:
//! - a pointer to the start of the slice
//! - the length of slice (in items)
//!
//! An extent slice is implemented using three pieces of data:
//! - a pointer to some outer slice
//! - the pffset of the start relative to the outer slice pointer
//! - the length of the slice
//!
//! The main difference is that when you create subslices using `.split_at`,
//! the pointer to the data is not updated, but some separate offset. This
//! means that an "extent slice" can be viewed as a form of relative pointer.
//!
//! The main benefit of this representation is it allows to compute the relative
//! offsets between an inner and outer slice without exposing the address of
//! the root pointer. Exposing the address of a pointer is not allowed in
//! `const` Rust as of version 1.92.0.
//!
//! Here is a concrete example of function that can't be written in const
//! Rust using regular slices.
//!
//! ```
//! struct Url<'input> {
//!   host: &'input [u8],
//!   // ...
//! }
//!
//! const fn parse_url(input: &[u8]) -> Url<'_> {
//!   // ...
//! #  Url {
//! #    host: input.split_at(8).1.split_at(11).0
//! #  }
//! }
//!
//! fn check() {
//!   let input: &[u8] = b"https://example.com/foo/bar";
//!   let host: &[u8] = parse_url(input).host;
//!   assert_eq!(host, b"example.com");
//!   // the line below is illegal in const Rust 1.92 (and likely for a long time)
//!   // because `pointer::addr` is not `const`
//!   let host_position: usize = host.as_ptr().addr() - input.as_ptr().addr();
//!   assert_eq!(host_position, 8);
//! }
//! ```
//!
//! If the `parse_url` is rewritten to use `XbstrRef` instead of `&[u8]`, then
//! the position of the host component can be retrieved in const Rust
//!
//! ```rust
//! use xslice::{Extent, XbstrRef};
//!
//! struct Url<'input> {
//!   host: XbstrRef<'input>,
//!   // ...
//! }
//!
//! const fn parse_url(input: XbstrRef<'_>) -> Url<'_> {
//!   // ...
//! #  Url {
//! #    host: input.sub_slice_checked(Extent::new(8, 11)).expect("extent is valid")
//! #  }
//! }
//!
//! const fn check() {
//!   let input: XbstrRef<'_> = XbstrRef::new(b"https://example.com/foo/bar");
//!   let host: XbstrRef<'_> = parse_url(input).host;
//!   assert!(matches!(host.get(), b"example.com"));
//!   // the line below is legal in const Rust 1.92
//!   let host_position: usize = host.extent().start_or_zero() - input.extent().start_or_zero();
//!   assert!(matches!(host_position, 8));
//! }
//!
//! check()
//! ```

mod extent;
mod xbstr;
mod xslice;
mod xstr;

pub use extent::Extent;
pub use xbstr::XbstrRef;
pub use xslice::XsliceRef;
pub use xstr::XstrRef;
