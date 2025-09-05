/// Create a EventQualifier from the given parameters
/// Mainly use to construct EventQualifier from the constant parameters to avoid runtime overhead.
/// # Examples
///
/// ```rust
/// use iolinke_device::event_qualifier_macro;
///
/// let event_qualifier = event_qualifier_macro!(1, 0, 0, 1);
/// assert_eq!(event_qualifier, 0b01000001);
/// ```
#[macro_export]
macro_rules! event_qualifier_macro {
    ($mode:expr, $type:expr, $source:expr, $instance:expr) => {
        (($mode as u8) << 6) | (($type as u8) << 4) | (($source as u8) << 3) | ($instance as u8)
    };
}
