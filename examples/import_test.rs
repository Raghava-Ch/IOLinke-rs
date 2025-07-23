//! Simple import test

use iolink_device_stack::{
    ApplicationLayerImpl,
    DlModeHandler,
    MessageHandler,
};

fn main() {
    let _app = ApplicationLayerImpl::new();
    let _dl = DlModeHandler::new();
    let _msg = MessageHandler::new();
    println!("Import test successful!");
}
