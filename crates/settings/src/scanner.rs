use crate::errors::SettingsReadError;
use crate::format::{RECORD_HEADER_SIZE, RecordHeader};
use crate::helpers::align_up;
use crate::page::{ActivePage, FLASH_PAGE_SIZE};
use crate::storage::FlashStorage;

pub struct FlashScanner<'a, F> {
    flash: &'a F
}

pub struct FoundRecord {
    pub page: ActivePage,
    pub offset: u32,
    pub sequence: u32,
    pub payload_len: u16,
}

impl<'a, F: FlashStorage> FlashScanner<'a, F> where SettingsReadError: From<<F as FlashStorage>::Error> {
    pub fn new(flash: &'a F) -> Self {
        Self { flash }
    }

    pub fn scan(&self) -> Result<Option<FoundRecord>, SettingsReadError> {
        let a = self.scan_page(ActivePage::A)?;
        let b = self.scan_page(ActivePage::B)?;

        match (a, b) {
            (None, None) => Ok(None),
            (Some(a), None) => Ok(Some(a)),
            (None, Some(b)) => Ok(Some(b)),
            (Some(a), Some(b)) if a.sequence < b.sequence => Ok(Some(a)),
            (Some(_a), Some(b)) => Ok(Some(b)),
        }
    }

    fn scan_page(&self, page: ActivePage) -> Result<Option<FoundRecord>, SettingsReadError> {
        let mut offset = 0;
        let mut newest = None;
        loop {
            if offset + RECORD_HEADER_SIZE > FLASH_PAGE_SIZE {
                break;
            }

            let mut header_buffer = [0u8; RECORD_HEADER_SIZE as usize];
            self.flash.read(page.start() + offset, &mut header_buffer)?;
            /*self.flash.lock::<Result<(), FlashError>>(|flash| {
                let mut flash = flash.borrow_mut();

                flash.blocking_read(page.start() + offset, &mut header_buffer)?;
                Ok(())
            })?;*/

            let candidate = RecordHeader::deserialize(&header_buffer)?;
            if candidate.is_valid() == false {
                break;
            }

            let record_size = RECORD_HEADER_SIZE + candidate.payload_length as u32;
            let aligned = align_up(record_size, 8);
            let candidate = FoundRecord {
                page,
                offset,
                sequence: candidate.sequence,
                payload_len: candidate.payload_length,
            };
            offset += aligned;

            newest = match newest {
                None => Some(candidate),
                Some(existing) if existing.sequence < candidate.sequence => Some(candidate),
                Some(existing) => Some(existing),
            }
        }

        Ok(newest)
    }
}
