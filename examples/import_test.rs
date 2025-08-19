use std::{thread::sleep, time::Duration};

use iolinke_device::*;


fn main() {
    let mut io_link_device = IoLinkDevice::new();

    loop {
        io_link_device.poll();
        // sleep(Duration::from_millis(100));
    }}
