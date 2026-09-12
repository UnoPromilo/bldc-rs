use core::array::TryFromSliceError;

pub fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

// TODO move to shared?
pub fn decode_u32(data: &[u8]) -> Result<u32, TryFromSliceError> {
    data.try_into().map(u32::from_le_bytes)
}

pub fn decode_u16(data: &[u8]) -> Result<u16, TryFromSliceError> {
    data.try_into().map(u16::from_le_bytes)
}
