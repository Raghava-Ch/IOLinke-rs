pub const REVISION_ID: u8 = crate::revision_id!(
    /*CONFIG:MAJOR_REVISION_ID*/ 0x09u8, /*ENDCONFIG*/
    /*CONFIG:MINOR_REVISION_ID*/ 0x4u8 /*ENDCONFIG*/
);

pub const VENDOR_ID: [u8; 2] = [
    /*CONFIG:VENDOR_ID_1*/ 0x01u8, /*ENDCONFIG*/
    /*CONFIG:VENDOR_ID_2*/ 0x01u8, /*ENDCONFIG*/
];
pub const DEVICE_ID: [u8; 3] = [
    /*CONFIG:DEVICE_ID_1*/ 0x01u8, /*ENDCONFIG*/
    /*CONFIG:DEVICE_ID_2*/ 0x01u8, /*ENDCONFIG*/
    /*CONFIG:DEVICE_ID_3*/ 0x01u8, /*ENDCONFIG*/
];
pub const FUNCTION_ID: [u8; 2] = [
    /*CONFIG:FUNCTION_ID_1*/ 0x01u8, /*ENDCONFIG*/
    /*CONFIG:FUNCTION_ID_2*/ 0x01u8, /*ENDCONFIG*/
];
pub const VENDOR_NAME: &str = /*CONFIG:VENDOR_NAME*/ "IOLinke" /*ENDCONFIG*/;
pub const PRODUCT_NAME: &str = /*CONFIG:PRODUCT_NAME*/ "IOLinke" /*ENDCONFIG*/;

// Parameter storage configuration must be defined in the IOLinke-Derived-CONFIG/src/device/vendor_specifics.rs file
// This is done because for current rust compiler limitations.
// As soon as rust compiler supports, storage configuration can be generalised and defined here.
// For now, this is the only way to define the parameter storage configuration.

/* Below these are non configurable components, helper components for the configuration */
#[macro_export]
macro_rules! revision_id {
    ($major_revision_id:expr, $minor_revision_id:expr) => {
        $major_revision_id << 4 | $minor_revision_id << 0
    };
}
