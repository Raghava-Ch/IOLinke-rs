/// Macro to configure basic IO-Link device parameters.
#[macro_export]
macro_rules! iolink_device_config {
    ($vendor_id:expr, $device_id:expr, $revision:expr) => {
        const VENDOR_ID: u16 = $vendor_id;
        const DEVICE_ID: u32 = $device_id;
        const REVISION: u8 = $revision;
    };
}

/// Macro to define IO-Link process data structure.
#[macro_export]
macro_rules! process_data {
    ($name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[repr(C, packed)]
        pub struct $name {
            $(pub $field: $type,)*
        }
    };
}

/// Macro to configure communication parameters.
#[macro_export]
macro_rules! comm_config {
    (baudrate: $baudrate:expr, min_cycle_time: $cycle_time:expr) => {
        const BAUDRATE: u32 = $baudrate;
        const MIN_CYCLE_TIME: u16 = $cycle_time;
    };
}

/// Macro to define parameter data.
#[macro_export]
macro_rules! parameter {
    ($index:expr, $subindex:expr, $access:expr, $type:ty, $default:expr) => {
        pub const INDEX: u16 = $index;
        pub const SUBINDEX: u8 = $subindex;
        pub const ACCESS: u8 = $access;
        pub type DataType = $type;
        pub const DEFAULT_VALUE: DataType = $default;
    };
}