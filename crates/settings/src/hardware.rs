use core::fmt::Error;
use crate::storage::FlashStorage;
use hardware::BoardFlashBank1;

pub struct BoardFlashStorage<'a> {
    flash: &'a BoardFlashBank1<'a>,
}

impl<'a> FlashStorage for BoardFlashStorage<'a> {
    type Error = hardware::FlashError;

    fn read(&self, offset: u32, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.flash.lock::<Result<(), Error>>(|flash| {
            let mut flash = flash.borrow_mut();

            flash.blocking_read(page.start() + offset, &mut header_buffer)?;
            Ok(())
        })
    }

    fn write(&self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn erase(&self, from: u32, to: u32) -> Result<(), Self::Error> {
        todo!()
    }
}
