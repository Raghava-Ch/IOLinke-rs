use crate::custom::IoLinkError;

/// See Table B.10 – DataStorageIndex assignments
/// Index 0x0003 and 0x0001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DsCommand {
    // Reserved = 0x00,
    /// {DS_UploadStart} 0x01
    UploadStart = 0x01,
    /// {DS_UploadEnd} 0x02
    UploadEnd = 0x02,
    /// {DS_DownloadStart} 0x03
    DownloadStart = 0x03,
    /// {DS_DownloadEnd} 0x04
    DownloadEnd = 0x04,
    /// {DS_Break} 0x05
    Break = 0x05,
    // 0x06 to 0xFF: Reserved
}

impl TryFrom<u8> for DsCommand {
    type Error = IoLinkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            x if x == Self::UploadStart as u8 => Self::UploadStart,

            x if x == Self::UploadEnd as u8 => Self::UploadEnd,
            x if x == Self::DownloadStart as u8 => Self::DownloadStart,
            x if x == Self::DownloadEnd as u8 => Self::DownloadEnd,
            x if x == Self::Break as u8 => Self::Break,
            _ => return Err(IoLinkError::InvalidData),
        })
    }
}
