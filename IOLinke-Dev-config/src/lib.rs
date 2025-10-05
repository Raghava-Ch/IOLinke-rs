//! # IOLinke-Dev-config Library
//!
//! This library provides configuration utilities and modules for IO-Link device development.
//! It is designed to be used in `no_std` environments and aims to facilitate the management
//! and interaction with IO-Link devices.
//!
//! ## Modules
//! - [`device`]: Contains types and functions related to IO-Link device configuration and management.
//!
//! ## Features
//! - `no_std` compatible
//! - Modular design for easy integration
//!
//! ## Usage
//! Add this crate as a dependency and import the required modules for device configuration tasks.
#![no_std]
#![warn(missing_docs)]

pub mod device;
