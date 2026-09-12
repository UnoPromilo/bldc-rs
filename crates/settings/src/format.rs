use crate::errors::SettingsReadError;
use crate::helpers::{decode_u16, decode_u32};
use crate::state::Settings;
use core::sync::atomic::Ordering;

const MAGIC: u32 = 0x53455454; // "SETT"
const VERSION: u16 = 1;

pub struct RecordHeader {
    magic: u32,
    version: u16,
    pub payload_length: u16,
    pub sequence: u32,
    crc: u32,
}

pub const RECORD_HEADER_SIZE: u32 = 16;

pub struct SettingsSnapshot {
    current_limit: u32,
    voltage_limit: u32,
}


impl Settings {
    fn snapshot(&self) -> SettingsSnapshot {
        SettingsSnapshot {
            current_limit: self.current_limit.load(Ordering::Relaxed),
            voltage_limit: self.voltage_limit.load(Ordering::Relaxed),
        }
    }
}

impl RecordHeader {
    pub fn deserialize(data: &[u8]) -> Result<Self, SettingsReadError> {
        Ok(Self {
            magic: decode_u32(&data[..4])?,
            version: decode_u16(&data[4..6])?,
            payload_length: decode_u16(&data[6..8])?,
            sequence: decode_u32(&data[8..12])?,
            crc: decode_u32(&data[12..16])?,
        })
    }

    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        buffer[..4].copy_from_slice(&self.magic.to_le_bytes());
        buffer[4..6].copy_from_slice(&self.version.to_le_bytes());
        buffer[6..8].copy_from_slice(&self.payload_length.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.sequence.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.crc.to_le_bytes());
        RECORD_HEADER_SIZE as usize
    }

    pub fn is_valid(&self) -> bool {
        self.magic == MAGIC && self.version == VERSION
    }
}

impl SettingsSnapshot {
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        buffer[..4].copy_from_slice(&self.current_limit.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.voltage_limit.to_le_bytes());
        8
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, SettingsReadError> {
        // TODO
        Ok(SettingsSnapshot {
            current_limit: 0,
            voltage_limit: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_header_serialization() {
        let header = RecordHeader {
            magic: MAGIC,
            version: VERSION,
            payload_length: 123,
            sequence: 456,
            crc: 789,
        };
        let mut buffer = [0u8; 16];
        let len = header.serialize(&mut buffer);
        assert_eq!(len, 16);
        assert_eq!(buffer[..4], MAGIC.to_le_bytes());
        assert_eq!(buffer[4..6], VERSION.to_le_bytes());
        assert_eq!(buffer[6..8], 123u16.to_le_bytes());
        assert_eq!(buffer[8..12], 456u32.to_le_bytes());
        assert_eq!(buffer[12..16], 789u32.to_le_bytes());
    }
}


