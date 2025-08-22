use bitfields::bitfield;
use iolinke_macros::bitfield_support;


/// See A.6.2 StatusCode type 1 (no details)
/// Figure A.21 shows the structure of this StatusCode.
#[bitfield(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StatusCodeType1 {
    #[bits(1)]
    event_details: u8,
    #[bits(1)]
    pd_valid: bool,
    #[bits(1)]
    reserved: bool,
    #[bits(5)]
    event_code: u8,
}

/// A.6.3 StatusCode type 2 (with details)
/// Figure A.22 shows the structure of the StatusCode type 2.
#[bitfield(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StatusCodeType2 {
    #[bits(1)]
    event_details: u8,
    #[bits(1)]
    __: u8,
    #[bits(1)]
    activated_event_slot1: u8,
    #[bits(1)]
    activated_event_slot2: u8,
    #[bits(1)]
    activated_event_slot3: u8,
    #[bits(1)]
    activated_event_slot4: u8,
    #[bits(1)]
    activated_event_slot5: u8,
    #[bits(1)]
    activated_event_slot6: u8,
}

/// See A.6.4 EventQualifier
/// The structure of the EventQualifier is shown in Figure A.24.
#[bitfield(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventQualifier {
    #[bits(2)]
    pub eq_mode: EventMode,
    #[bits(2)]
    pub eq_type: EventType,
    #[bits(1)]
    pub eq_source: EventSource,
    #[bits(3)]
    pub eq_instance: EventInstance,
}


/// EventQualifier INSTANCE field (Bits 0..=2)
/// See Table A.17 – Values of INSTANCE
#[bitfield_support]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventInstance {
    Unknown = 0,
    // Reserved = 1..=3,
    Application = 4,
    System = 5,
    // Reserved = 6..=7,
}

impl EventInstance {
    pub const fn new() -> Self {
        Self::Unknown
    }
}

/// EventQualifier SOURCE field (Bits 3)
/// See Table A.18 – Values of SOURCE
#[bitfield_support]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    /// Device (remote)
    Device = 0,
    /// Master/Port
    Master = 1,
}

impl EventSource {
    pub const fn new() -> Self {
        Self::Device
    }
}

/// EventQualifier TYPE field (Bits 4..=5)
/// See Table A.19 – Values of TYPE
#[bitfield_support]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    // Reserved = 0
    Notification = 1,
    Warning = 2,
    Error = 3,
}

impl EventType {
    pub const fn new() -> Self {
        Self::Notification
    }
}

/// EventQualifier MODE field (Bits 6..=7)
/// See Table A.20 – Values of MODE
#[bitfield_support]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventMode {
    // Reserved = 0,
    SingleShot = 1,
    Disappears = 2,
    Appears = 3,
}

impl EventMode {
    pub const fn new() -> Self {
        Self::SingleShot
    }
}