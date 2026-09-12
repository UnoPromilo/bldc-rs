use core::array::TryFromSliceError;
#[cfg(feature = "hardware-support")]
use hardware::FlashError;

#[derive(Debug)]
pub enum SettingsReadError {
    TryFromSlice(TryFromSliceError),
    #[cfg(feature = "hardware-support")]
    FlashError(FlashError),
}

impl From<TryFromSliceError> for SettingsReadError {
    fn from(value: TryFromSliceError) -> Self {
        SettingsReadError::TryFromSlice(value)
    }
}

#[cfg(feature = "hardware-support")]
impl From<FlashError> for SettingsReadError {
    fn from(value: FlashError) -> Self {
        SettingsReadError::FlashError(value)
    }
}

#[cfg(test)]
impl From<()> for SettingsReadError{
    fn from(_value: ()) -> Self {
        unimplemented!()
    }
}