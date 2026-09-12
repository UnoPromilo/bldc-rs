use crate::errors::SettingsReadError;
use crate::helpers::align_up;
use crate::page::ActivePage;
use crate::scanner::FlashScanner;
use crate::storage::FlashStorage;

pub struct SettingsManager<'a, F> {
    flash: &'a F,
    state: FlashState,
}

struct FlashState {
    active_page: ActivePage,
    next_sequence: u32,
    next_write_offset: u32,
}

impl<'a, F: FlashStorage> SettingsManager<'a, F> {
    pub fn new(flash: &'a F) -> Result<Self, SettingsReadError>
    where
        SettingsReadError: From<<F as FlashStorage>::Error>,
    {
        let newest = {
            let scanner = FlashScanner::new(flash);
            scanner.scan()?
        };

        let state = match newest {
            None => FlashState::default(),
            Some(record) => FlashState {
                active_page: record.page,
                next_sequence: record.sequence + 1,
                next_write_offset: record.offset + align_up(record.payload_len as u32, 8),
            },
        };

        Ok(Self { flash, state })
    }
}

impl Default for FlashState {
    fn default() -> Self {
        FlashState {
            active_page: ActivePage::A,
            next_sequence: 1,
            next_write_offset: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::MockFlash;

    #[test]
    fn new_uses_default_state_when_flash_is_empty() {
        let flash = MockFlash::new(4096);

        let manager = SettingsManager::new(&flash).unwrap();

        assert_eq!(manager.state.active_page, ActivePage::A);
        assert_eq!(manager.state.next_sequence, 1);
        assert_eq!(manager.state.next_write_offset, 0);
    }
}