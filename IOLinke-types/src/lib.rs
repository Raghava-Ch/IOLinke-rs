#![no_std]
#![warn(missing_docs)]
//! # IOLinke-types
//!
//! This crate provides core types and modules for working with IO-Link communication in Rust.
//! It is designed to be used in `no_std` environments, making it suitable for embedded systems.
//!
//! ## Modules
//! - [`custom`]: Contains custom type definitions and utilities specific to IO-Link.
//! - [`frame`]: Provides structures and functions for handling IO-Link frames.
//! - [`handlers`]: Includes handler traits and implementations for IO-Link operations.
//! - [`page`]: Defines types and logic for managing IO-Link pages.
//!
//! ## Features
//! - `no_std` support for embedded applications.
//! - Modular organization for easy extension and maintenance.
//!
//! ## Usage
//! Import the required modules and types to interact with IO-Link devices and protocols.
//!
//! ## License
//! This crate is distributed under the terms of the MIT license.

pub mod custom;
pub mod frame;
pub mod handlers;
pub mod page;
