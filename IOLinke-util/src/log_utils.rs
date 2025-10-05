//! Logging utilities for IO-Link Device Stack.
//!
//! This module provides macros and helper functions for logging
//! function calls, state transitions, and events within the stack.
//!
//! # Features
//!
//! - Conditional logging based on the `std` feature
//! - Timestamped log output (when `std` is enabled)
//! - Macro for standardized function call logging
//!
//! # Usage
//!
//! Use the [`log_fn_call!`] macro to log function calls and state transitions:
//!
//! ```ignore
//! log_fn_call!(
//!     "IoLinkDevice",
//!     "poll",
//!     "Idle",
//!     "Active",
//!     "Polling all protocol layers"
//! );
//! ```
//!
//! When the `std` feature is enabled, logs are printed to the standard output
//! with a timestamp. In `no_std` environments, the macro does nothing.
//!
//! # Example Output
//!
//! ```text
//! [1681234567] [IoLinkDevice] [poll] [Idle] [Active] Polling all protocol layers
//! ```

/// Log a function call and state transition.
///
/// This macro logs a function call and the transition from one state to another.
/// It is used to track the flow of execution and state changes within the stack.
///
/// # Arguments
///
/// - `$module`: The name of the module or component making the call.
/// - `$event`: The name of the function or event being logged.
/// - `$source_state`: The state the system was in before the call.
/// - `$target_state`: The state the system is in after the call.
/// - `$details`: Additional details or context about the call.
///
/// # Example
///
/// ```ignore
/// log_fn_call!(
///     "IoLinkDevice",
///     "poll",
///     "Idle",
///     "Active",
///     "Polling all protocol layers"
/// );
/// ```
#[macro_export]
macro_rules! log_fn_call {
    (
        $module:expr,
        $event:expr,
        $source_state:expr,
        $target_state:expr,
        $details:expr
    ) => {
        #[cfg(feature = "std")]
        {
            let now = std::time::SystemTime::now();
            let timestamp = now
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let timestamp = timestamp.to_string();

            std::println!(
                "[{}] [{}] [{}] [{}] [{}] {}",
                timestamp,
                $module,
                $event,
                $source_state,
                $target_state,
                $details
            );
        }
    };
}

/// Log a function call and state transition.
///
/// This macro logs a function call and the transition from one state to another.
/// It is used to track the flow of execution and state changes within the stack.
///
/// # Arguments
///
/// - `$module`: The name of the module or component making the call.
/// - `$event`: The name of the function or event being logged.
/// - `$source_state`: The state the system was in before the call.
/// - `$target_state`: The state the system is in after the call.
/// - `$details`: Additional details or context about the call.
///
/// # Example
///
/// ```ignore
/// log_state_transition!(
///     "IoLinkDevice",
///     "poll",
///     "Idle",
///     "Active",
///     "Polling all protocol layers"
/// );
/// ```
#[macro_export]
macro_rules! log_state_transition {
    (
        $module:expr,
        $event:expr,
        $source_state:expr,
        $target_state:expr,
        $details:expr
    ) => {
        #[cfg(feature = "std")]
        {
            let now = std::time::SystemTime::now();
            let timestamp = now
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let timestamp = timestamp.to_string();

            std::println!(
                "[{}] [{}] [{}] [{:?}] [{:?}] [{:?}]",
                timestamp,
                $module,
                $event,
                $source_state,
                $target_state,
                $details
            );
        }
    };
}

/// Log a state transition error.
#[macro_export]
macro_rules! log_state_transition_error {
    (
        $module:expr,
        $event:expr,
        $current_state:expr,
        $details:expr
    ) => {
        #[cfg(feature = "std")]
        {
            let now = std::time::SystemTime::now();
            let timestamp = now
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let timestamp = timestamp.to_string();

            std::eprintln!(
                "[{}] [{}] [{}] [{:?}] [{:?}]",
                timestamp,
                $module,
                $event,
                $current_state,
                $details
            );
        }
    };
}
