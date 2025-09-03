// Test module for iolinke_device crate
// This file organizes all test modules

use std::time::Duration;

use iolinke_device::{
    DataStorageIndexSubIndex, DeviceParametersIndex, MsequenceCapability, SubIndex, config,
    test_utils::{self, IsduService, TestDeviceMode},
};

pub mod isdu_tests;
pub mod preop_tests;
pub mod startup_tests;

#[test]
fn mock_test_device_operations() {
    
}
