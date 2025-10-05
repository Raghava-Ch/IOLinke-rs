//! FFI (Foreign Function Interface) module for IOLinke-Bindings.
//!
//! This module provides interoperability between Rust and C code,
//! exposing necessary bindings and types for external usage.
//!
//! # Features
//! - `#![no_std]` for embedded and constrained environments.
//! - Contains submodules for C bindings.
//!
//! # Usage
//! Import the required types and functions from the `c` submodule as needed.
#![no_std]
#![warn(missing_docs)]

pub mod c;
