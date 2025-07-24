Create a Rust project named `iolink_device_stack` to implement a modular, maintainable, and portable IO-Link Device/Slave stack compliant with IO-Link Specification Version 1.1.4 (June 2024, Order No: 10.002).

---

### 🧱 Core Architecture

1. **Language & Platform**
   - Written in Rust (`#![no_std]`) for embedded microcontroller targets.
   - Must compile for bare-metal targets (e.g., Cortex-M).
   - Platform portability through HAL traits.
   - Expose a C-compatible interface via `#[no_mangle] extern "C"`.

2. **State Machines**
   - Use the **latest version** of the `smlang` crate to define all protocol state machines.
   - Each module must use polling (`poll()` method) instead of interrupts or async.
   - State machines must be implemented for the following 12 components:
     - DL-Mode Handler
     - Message Handler
     - Process Data Handler
     - On-request Data Handler
     - ISDU Handler
     - Command Handler
     - Event Handler
     - Application Layer
     - Event State Machine
     - System Management
     - Parameter Manager
     - Data Storage

3. **Application Layer Public API (Based on Section 8.4)**
    - “Define a Rust trait ApplicationLayer that includes all request/indication methods described in IO-Link Specification Section 8.4. This includes AL_GetInput_req, AL_SetOutput_req, ISDU access methods, event/control indications, and event history access, with all supporting enums and data types.”

4. **Physical Layer Public API (Based on Section 5.2, Figure 13, )**
    - “Define a Rust trait PhysicalLayer that encapsulates low-level UART/PHY access as described in Section 5.2 and Figure 13, including PL_SetMode, PL_Transfer, PL_WakeUp, and PL_Status operations. Use appropriate enums for status, mode, and error reporting.”

5. **C Interoperability (FFI)**
   - Expose all the interface (Refer the ApplicationLayer and PhysicalLayer) functions for C integration:
   - Generate C headers using `cbindgen`.

6. **Utilities**
   - Use nom crate to parse the IO-Link frames

---

### 📁 Project Structure
📁 iolink_device_stack/
├── src/
│   ├── lib.rs
│   ├── hal.rs
│   ├── types.rs
│   ├── ffi.rs
│   ├── application.rs
│   ├── dl_mode.rs
│   ├── message.rs
│   ├── process_data.rs
│   ├── on_request.rs
│   ├── isdu.rs
│   ├── command.rs
│   ├── event.rs
│   ├── event_sm.rs
│   ├── sm.rs
│   ├── parameter.rs
│   ├── storage.rs
│   └── test_utils.rs


---

### 📚 Documentation
All APIs should be documented with inline Rust doc comments (///) and explicitly link to the relevant section in the IO-Link spec. For example:
/// See IO-Link v1.1.4 Section 8.4.2.1 for AL_GetInput_req behavior.

---

### 📦 Dependency crates
smlang
heapless
embedded-hal
cstr_core
log
nom
