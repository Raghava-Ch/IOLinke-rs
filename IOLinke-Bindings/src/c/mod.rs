//! This module provides the core C bindings for the IO-Link project.
//!
//! It exposes submodules for application logic (`app`), physical layer interactions (`phy`),
//! common type definitions (`types`), and customizable hooks (`hooks`).
//!
//! These bindings facilitate interoperability between Rust and C components within the IO-Link ecosystem.

pub mod app;
pub mod hooks;
pub mod phy;
pub mod types;
