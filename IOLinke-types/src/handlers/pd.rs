//! # IO-Link Process Data (PD) Handler Module
//!
//! This module provides traits and types for handling Process Data (PD) input and output operations
//! in IO-Link devices. It defines interfaces for updating input data, transporting output data, and
//! signaling the end of PD cycles, as well as structures for representing process data and handler states.
//!
//! ## Key Traits
//! - [`DlPDInputUpdate`]: Handles PD input update requests.
//! - [`DlPDOutputTransportInd`]: Handles PD output transport indications and cycle completion.
//!
//! ## Key Types
//! - [`ProcessData`]: Structure representing input/output process data and validity.
//! - [`PdConfState`]: Configuration state for the PD handler.
//!
//! ## Specification Reference
//! - IO-Link v1.1.4, Section 8.4.2 (Process Data)
//! - Figure 47 – State machine of the Device Process Data handler
//!
//! This module is intended for use in IO-Link device implementations to manage process data
//! communication and state transitions in compliance with the protocol.

use crate::custom::IoLinkResult;
use heapless::Vec;
use iolinke_dev_config::device as dev_config;

/// Maximum PD input length based on the device configuration
/// This is determined by the configured PD input length in bytes as defined in the
/// device configuration.
pub const PD_INPUT_LENGTH: usize =
    dev_config::process_data::config_pd_in_length_in_bytes() as usize;

/// Maximum PD output length based on the device configuration
/// This is determined by the configured PD output length in bytes as defined in the
/// device configuration.
pub const PD_OUTPUT_LENGTH: usize =
    dev_config::process_data::config_pd_out_length_in_bytes() as usize;

/// Trait representing the DL_PDInputUpdate service handler.
pub trait DlPDInputUpdate {
    /// DL_PDInputUpdate service trait for updating input data (Process Data from Device to Master)
    ///
    /// The `DlPDInputUpdate` trait provides an interface for the DL_PDInputUpdate service, which is used
    /// by the device's application layer to update input data on the data link layer.
    ///
    /// # Methods
    ///
    /// Sends a DL_PDInputUpdate request to update the input data on the data link layer.
    ///
    /// # Arguments
    /// * `length` - The length of the input data.
    /// * `input_data` - A slice containing the process data provided by the application layer.
    ///
    /// # Returns
    /// * `IoLinkResult<()>` - Returns `Ok(())` if the service has been executed successfully.
    ///   On failure, returns an error containing error information:
    ///     - `NO_COMM`: No communication available.
    ///     - `STATE_CONFLICT`: Service unavailable within current state.
    ///
    /// # Notes
    /// The service-specific parameters are transmitted in the argument.
    /// The result indicates whether the data link layer is in a state permitting data transmission.
    fn dl_pd_input_update_req(&mut self, length: u8, input_data: &[u8]) -> IoLinkResult<()>;
}

/// Trait representing the DL_PDOutputTransport service handler.
/// This trait is used by the data link layer on the Device to transfer output Process Data
/// to the application layer.
pub trait DlPDOutputTransportInd {
    /// DL_PDOutputTransport service indication for output process data transfer (Master to Device)
    ///
    /// This method is called by the data link layer on the Device to transfer the content of output
    /// Process Data to the application layer. The parameters of the service primitive are:
    /// - Argument: Service-specific parameters transmitted in the argument.
    /// - OutputData: Contains the Process Data to be transmitted to the application layer.
    ///
    /// # Arguments
    /// * `pd_out` - A reference to the output Process Data (octet string) to be delivered to the application layer.
    ///
    /// # Returns
    /// * `IoLinkResult<()>` - Returns `Ok(())` if the service has been executed successfully.
    ///   On failure, returns an error containing error information.
    fn dl_pd_output_transport_ind(
        &mut self,
        pd_out: &Vec<u8, PD_OUTPUT_LENGTH>,
    ) -> IoLinkResult<()>;

    /// DL_PDCycle service indication for signaling the end of a Process Data cycle.
    ///
    /// This method is called by the data link layer to notify the application layer that a Process Data cycle has completed.
    /// The service does not require any parameters.
    ///
    /// # Returns
    /// * `IoLinkResult<()>` - Returns `Ok(())` if the indication has been processed successfully.
    ///   On failure, returns an error containing error information.
    fn dl_pd_cycle_ind(&mut self) -> IoLinkResult<()>;
}

/// Process data input/output structure
/// See IO-Link v1.1.4 Section 8.4.2
#[derive(Debug, Default, Clone)]
pub struct ProcessData {
    /// Input data from device
    pub input: Vec<u8, PD_INPUT_LENGTH>,
    /// Output data to device
    pub output: Vec<u8, PD_OUTPUT_LENGTH>,
    /// Data validity flag
    pub valid: bool,
}

/// All the Process Data Handler configuration states used
/// See Figure 47 – State machine of the Device Process Data handler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdConfState {
    /// (Handler changed to the ACTIVE state)
    Active,
    /// (Handler changed to the INACTIVE state)
    Inactive,
}
