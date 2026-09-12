pub trait FlashStorage {
    type Error;

    fn read(&self, offset: u32, buffer: &mut [u8]) -> Result<(), Self::Error>;

    fn write(&self, offset: u32, data: &[u8]) -> Result<(), Self::Error>;
}

#[cfg(test)]
pub mod mock {
    use alloc::vec;
    use super::*;

    #[derive(Default)]
    pub struct MockFlash {
        memory: vec::Vec<u8>,
    }

    impl MockFlash {
        pub fn new(size: usize) -> Self {
            Self {
                memory: vec![0xFF; size],
            }
        }

        pub fn write_bytes(&mut self, offset: usize, data: &[u8]) {
            self.memory[offset..offset + data.len()].copy_from_slice(data);
        }
    }

    impl FlashStorage for MockFlash {
        type Error = ();

        fn read(&self, offset: u32, buffer: &mut [u8]) -> Result<(), Self::Error> {
            buffer.copy_from_slice(
                &self.memory[offset as usize..offset as usize + buffer.len()],
            );
            Ok(())
        }

        fn write(&self, _: u32, _: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }
}