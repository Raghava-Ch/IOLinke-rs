//! Mock physical layer implementation for testing IO-Link device stack

use crate::*;
use crate::{IoLinkDevice, IoLinkError, IoLinkMode, IoLinkResult, PhysicalLayerReq, Timer};
use crate::pl::physical_layer::PhysicalLayerInd;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::Instant;
use super::types::ThreadMessage;

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

    /// Transfer the received data to the IO-Link device
    pub fn transfer_ind(
        &mut self,
        rx_buffer: &[u8],
        io_link_device: Arc<Mutex<IoLinkDevice>>,
    ) -> IoLinkResult<()> {
        self.rx_data.clear();
        self.rx_data.extend_from_slice(rx_buffer);
        let mut io_link_device_lock = io_link_device.lock().unwrap();
        for rx_buffer_byte in rx_buffer {
            let _ = io_link_device_lock.pl_transfer_ind(self, *rx_buffer_byte);
        }
        Ok(())
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

impl PhysicalLayerReq for MockPhysicalLayer {
    fn pl_set_mode_req(&mut self, mode: IoLinkMode) -> IoLinkResult<()> {
        self.mode = mode;
        Ok(())
    }

    fn pl_transfer_req(&mut self, tx_data: &[u8]) -> IoLinkResult<usize> {
        self.tx_data.clear();
        self.tx_data.extend_from_slice(tx_data);
        self.mock_to_usr_tx
            .send(ThreadMessage::TxData(self.tx_data.clone()))
            .unwrap();
        let tx_data_len = tx_data.len();
        Ok(tx_data_len)
    }

    fn stop_timer(&self, timer: Timer) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        for timer_state in timers.iter_mut() {
            if timer_state.timer_id == timer {
                timer_state.stop();
                break;
            }
        }
        Ok(())
    }

    fn start_timer(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
        let mut timers = self.timers.lock().unwrap();
        let timer_state = MockTimerState::new(timer, duration_us);
        timers.push(timer_state);
        Ok(())
    }

    fn restart_timer(&self, timer: Timer, duration_us: u32) -> IoLinkResult<()> {
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
