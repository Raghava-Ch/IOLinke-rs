
/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_failure_code {
    () => {
        0x4
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_success_code {
    () => {
        0x5
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_failure_code {
    () => {
        0xC
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_success_code {
    () => {
        0xD
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_request_index_code {
    () => {
        0x9
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_request_index_subindex_code {
    () => {
        0xA
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_read_request_index_index_subindex_code {
    () => {
        0xB
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_request_index_code {
    () => {
        0x1
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_request_index_subindex_code {
    () => {
        0x2
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_write_request_index_index_subindex_code {
    () => {
        0x3
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_extended_length_code {
    () => {
        0x1
    };
}

/// See table A.13 – ISDU syntax
/// Table A.13 specifies the syntax of the ISDUs. ErrorType can be found in Annex C.
#[macro_export]
macro_rules! isdu_no_service {
    () => {
        0x0
    };
}