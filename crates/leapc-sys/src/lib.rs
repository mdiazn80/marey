//! Raw LeapC bindings, generated at build time from the installed SDK header.
//!
//! See `docs/adr/0001-leapc-bindings.md`. Use the `leapc` crate instead of this one.

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    clippy::all,
    rustdoc::all
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
