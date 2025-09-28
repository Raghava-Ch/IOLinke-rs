//! Mock physical layer implementation for testing IO-Link device stack

use crate::mock_app_layer::MockApplicationLayer;

use super::types::ThreadMessage;
use iolinke_device::{IoLinkDevice, PhysicalLayerReq, Timer};
use iolinke_types::custom::IoLinkResult;
use iolinke_types::handlers::sm::IoLinkMode;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use core::result::Result::Ok;
use std::vec::Vec;

/// Mock timer state for tracking timer expiration
pub struct MockTimerState {
    timer_id: Timer,
    start_time: Instant,
    duration_us: u32,
    active: bool,
}

impl MockTimerState {
    fn new(timer_id: Timer, duration_us: u32) -> Self {
        Self {
            timer_id,
            start_time: Instant::now(),
            duration_us,
            active: true,
        }
    }

    fn is_expired(&self) -> bool {
        if !self.active {
            return false;
        }
        let elapsed = self.start_time.elapsed();
        let elapsed_us = elapsed.as_micros() as u32;
        elapsed_us >= self.duration_us
    }

    fn restart(&mut self, duration_us: u32) {
        self.start_time = Instant::now();
        self.duration_us = duration_us;
        self.active = true;
    }

    fn stop(&mut self) {
        self.active = false;
    }
}

/// Mock physical layer implementation for testing
pub struct MockPhysicalLayer {
    mode: IoLinkMode,
    tx_data: Vec<u8>,
    rx_data: Vec<u8>,
    timers: Arc<Mutex<Vec<MockTimerState>>>,
    mock_to_usr_tx: Sender<ThreadMessage>,
}

impl MockPhysicalLayer {
    /// Create a new MockPhysicalLayer
    ///
    /// # Arguments
    ///
    /// * `mock_to_usr_tx` - A sender for messages to the user thread
    ///
    /// # Returns
    ///
    /// A new MockPhysicalLayer
    ///
    pub fn new(mock_to_usr_tx: Sender<ThreadMessage>) -> Self {
        Self {
            mode: IoLinkMode::Inactive,
            tx_data: Vec::new(),
            rx_data: Vec::new(),
            timers: Arc::new(Mutex::new(Vec::new())),
            mock_to_usr_tx,
        }
    }

    pub fn set_rx_data_from_slice(&mut self, data: &[u8]) {
        self.rx_data.clear();
        self.rx_data.extend_from_slice(data);
    }

    pub fn timer_expired(&mut self, timer: Timer) {
        println!("Timer expired: {:?}", timer);
        // Here you can add any specific logic needed when a timer expires
        // For example, triggering state transitions or error handling
    }

    /// Check if any timers have expired and call timer_expired for them
    pub fn check_timers(&mut self) {
        let expired_timers = {
            let mut timers = self.timers.lock().unwrap();
            let mut expired_timers = Vec::new();
            let mut i = 0;

            // Find expired timers
            while i < timers.len() {
                if timers[i].is_expired() {
                    expired_timers.push(timers[i].timer_id);
                    timers.remove(i);
                } else {
                    i += 1;
                }
            }
            expired_timers
        };

        // Call timer_expired for each expired timer (after releasing the lock)
        for timer_id in expired_timers {
            self.timer_expired(timer_id);
        }
    }
}

/// Transfer the received data to the IO-Link device
pub fn transfer_ind(
    rx_buffer: &[u8],
    io_link_device_lock: &mut IoLinkDevice<MockPhysicalLayer, MockApplicationLayer>,
) -> IoLinkResult<()> {
    for rx_buffer_byte in rx_buffer {
        let _ = io_link_device_lock.pl_transfer_ind(*rx_buffer_byte);
    }
    Ok(())
}

impl PhysicalLayerReq for MockPhysicalLayer {
    fn pl_set_mode_req(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.mode = mode;
        Ok(())
    }

    fn pl_transfer_req(&mut self, tx_data: &[u8]) -> IoLinkResult<()> {
        self.tx_data.clear();
        self.tx_data.extend_from_slice(tx_data);
        self.mock_to_usr_tx
            .send(ThreadMessage::TxData(self.tx_data.clone()))
            .unwrap();
        let _tx_data_len = tx_data.len();
        Ok(())
    }

    fn pl_stop_timer_req(&self, timer: Timer) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        for timer_state in timers.iter_mut() {
            if timer_state.timer_id == timer {
                timer_state.stop();
                break;
            }
        }
        Ok(())
    }

    fn pl_start_timer_req(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        let timer_state = MockTimerState::new(timer, duration_us);
        timers.push(timer_state);
        Ok(())
    }

    fn pl_restart_timer_req(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        for timer_state in timers.iter_mut() {
            if timer_state.timer_id == timer {
                timer_state.restart(duration_us);
                return Ok(());
            }
        }
        // If timer doesn't exist, create it
        let timer_state = MockTimerState::new(timer, duration_us);
        timers.push(timer_state);
        Ok(())
    }
}
