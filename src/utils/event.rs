/// Create a EventQualifier from the given parameters
/// Mainly use to construct EventQualifier from the constant parameters to avoid runtime overhead.
/// # Examples
///
/// ```rust
/// use iolinke_device::utils::event::event_qualifier_macro;
///
/// let event_qualifier = event_qualifier_macro!(storage::event_memory::EventMode::SingleShot, storage::event_memory::EventType::Notification, storage::event_memory::EventSource::Device, storage::event_memory::EventInstance::System);
/// assert_eq!(event_qualifier, 0b00000000000000000000000000000000);
/// ```
#[macro_export]
macro_rules! event_qualifier_macro {
    ($mode:expr, $type:expr, $source:expr, $instance:expr) => {
        (($mode as u8) << 6) | (($type as u8) << 4) | (($source as u8) << 3) | ($instance as u8)
    };
}