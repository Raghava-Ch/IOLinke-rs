
/// Transmission rate configuration for IO-Link communication.
///
/// The transmission rate determines the baud rate used for
/// master-device communication.
///
/// # Specification Reference
///
/// - IO-Link v1.1.4 Section 5.2.2: Communication Modes
/// - Table 5.1: Communication mode characteristics
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransmissionRate {
    /// COM1 mode - 1200 baud effective rate
    Com1,
    /// COM2 mode - 2400 baud effective rate
    Com2,
    /// COM3 mode - 4800 baud effective rate
    Com3,
}

impl Default for TransmissionRate {
    /// Default transmission rate is COM1 (1200 baud).
    fn default() -> Self {
        Self::Com1
    }
}

impl TransmissionRate {
    /// Returns the baud rate for the given UART mode.
    ///
    /// # IO-Link Specification Reference
    ///
    /// - Section A.3.2: Bit time
    /// - Equation (A.2): TBIT = 1 / (transmission rate)
    /// - Table 9: Values for TBIT
    ///
    /// The baud rate is the number of bits transmitted per second. This value is used for timing calculations in the IO-Link protocol.
    ///
    /// # Returns
    ///
    /// Baud rate in bits per second (bps) for the selected UART mode.
    ///
    /// # Example
    ///
    /// ```
    /// let baud_rate = IoLinkUartMode::Com1.get_baud_rate();
    /// assert_eq!(baud_rate, 4800); // 4.8 kbaud
    /// ```
    pub const fn get_baud_rate(self) -> u32 {
        match self {
            Self::Com1 => 4800,
            Self::Com2 => 38400,
            Self::Com3 => 230400,
        }
    }

    /// Returns the bit time (T_BIT) in microseconds for the given UART mode.
    ///
    /// # IO-Link Specification Reference
    ///
    /// - Section A.3.2: Bit time
    /// - Equation (A.2): TBIT = 1 / (transmission rate)
    /// - Table 9: Values for TBIT
    ///
    /// The bit time (T_BIT) is the time required to transmit a single bit and is the inverse of the
    /// transmission rate. This value is used for timing calculations in the IO-Link protocol.
    ///
    /// # Returns
    ///
    /// Bit time in microseconds (µs) for the selected UART mode.
    ///
    /// # Example
    ///
    /// ```
    /// let tbit = IoLinkUartMode::Com1.get_t_bit_in_us();
    /// assert_eq!(tbit, 20833); // 1/4800 s = 20833 µs
    /// ```
    pub const fn get_t_bit_in_us(self) -> u32 {
        match self {
            Self::Com1 => 20833,
            Self::Com2 => 2604,
            Self::Com3 => 434,
        }
    }
}