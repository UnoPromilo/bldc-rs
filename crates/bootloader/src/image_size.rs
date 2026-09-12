#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DownloadError {
    EmptyChunk,
    ChunkTooLarge,
    ImageTooLarge,
    DataAfterShortBlock,
    ImageTooSmall,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingChunk {
    next_size: usize,
    is_short: bool,
}

pub struct DownloadTracker<const BLOCK_SIZE: usize> {
    maximum_size: usize,
    received_size: usize,
    received_short_block: bool,
    failed: bool,
}

const fn checked_image_size(
    received_size: usize,
    chunk_size: usize,
    maximum_size: usize,
) -> Option<usize> {
    match received_size.checked_add(chunk_size) {
        Some(new_size) if new_size <= maximum_size => Some(new_size),
        _ => None,
    }
}

const _: () = assert!(matches!(
    checked_image_size(196 * 1024 - 4096, 4096, 196 * 1024),
    Some(200704)
));
const _: () = assert!(checked_image_size(196 * 1024, 1, 196 * 1024).is_none());

impl<const BLOCK_SIZE: usize> DownloadTracker<BLOCK_SIZE> {
    pub const fn new(maximum_size: usize) -> Self {
        Self {
            maximum_size,
            received_size: 0,
            received_short_block: false,
            failed: false,
        }
    }

    pub fn reset(&mut self) {
        self.received_size = 0;
        self.received_short_block = false;
        self.failed = false;
    }

    pub fn check_chunk(&mut self, chunk_size: usize) -> Result<PendingChunk, DownloadError> {
        let result = if self.failed {
            Err(DownloadError::Failed)
        } else if chunk_size == 0 {
            Err(DownloadError::EmptyChunk)
        } else if chunk_size > BLOCK_SIZE {
            Err(DownloadError::ChunkTooLarge)
        } else if self.received_short_block {
            Err(DownloadError::DataAfterShortBlock)
        } else {
            checked_image_size(self.received_size, chunk_size, self.maximum_size)
                .map(|next_size| PendingChunk {
                    next_size,
                    is_short: chunk_size < BLOCK_SIZE,
                })
                .ok_or(DownloadError::ImageTooLarge)
        };

        if result.is_err() {
            self.failed = true;
        }

        result
    }

    pub fn commit(&mut self, chunk: PendingChunk) {
        self.received_size = chunk.next_size;
        self.received_short_block = chunk.is_short;
    }

    pub fn fail(&mut self) {
        self.failed = true;
    }

    pub fn finish(&self) -> Result<usize, DownloadError> {
        if self.failed {
            return Err(DownloadError::Failed);
        }
        if self.received_size < 8 {
            return Err(DownloadError::ImageTooSmall);
        }

        Ok(self.received_size)
    }
}

#[cfg(test)]
mod tests {
    use super::{DownloadError, DownloadTracker};

    #[test]
    fn accepts_exact_maximum_size() {
        let mut tracker = DownloadTracker::<4096>::new(196 * 1024);
        for _ in 0..49 {
            let chunk = tracker.check_chunk(4096).unwrap();
            tracker.commit(chunk);
        }
        assert_eq!(tracker.finish(), Ok(196 * 1024));
    }

    #[test]
    fn rejects_oversized_image() {
        let mut tracker = DownloadTracker::<4096>::new(4096);
        let chunk = tracker.check_chunk(4096).unwrap();
        tracker.commit(chunk);
        assert_eq!(tracker.check_chunk(1), Err(DownloadError::ImageTooLarge));
        assert_eq!(tracker.finish(), Err(DownloadError::Failed));
    }

    #[test]
    fn rejects_data_after_a_short_block() {
        let mut tracker = DownloadTracker::<4096>::new(8192);
        let chunk = tracker.check_chunk(1024).unwrap();
        tracker.commit(chunk);
        assert_eq!(
            tracker.check_chunk(1024),
            Err(DownloadError::DataAfterShortBlock)
        );
        assert_eq!(tracker.finish(), Err(DownloadError::Failed));
    }

    #[test]
    fn rejects_empty_and_oversized_chunks() {
        let mut tracker = DownloadTracker::<4096>::new(8192);
        assert_eq!(tracker.check_chunk(0), Err(DownloadError::EmptyChunk));
        assert_eq!(tracker.finish(), Err(DownloadError::Failed));

        tracker.reset();
        assert_eq!(tracker.check_chunk(4097), Err(DownloadError::ChunkTooLarge));
        assert_eq!(tracker.finish(), Err(DownloadError::Failed));
    }

    #[test]
    fn rejects_an_image_without_a_vector_table() {
        let mut tracker = DownloadTracker::<4096>::new(8192);
        let chunk = tracker.check_chunk(7).unwrap();
        tracker.commit(chunk);
        assert_eq!(tracker.finish(), Err(DownloadError::ImageTooSmall));
    }

    #[test]
    fn reset_starts_a_new_download() {
        let mut tracker = DownloadTracker::<4096>::new(8192);
        let chunk = tracker.check_chunk(1024).unwrap();
        tracker.commit(chunk);
        assert_eq!(
            tracker.check_chunk(1024),
            Err(DownloadError::DataAfterShortBlock)
        );
        tracker.reset();

        let chunk = tracker.check_chunk(4096).unwrap();
        tracker.commit(chunk);
        assert_eq!(tracker.finish(), Ok(4096));
    }
}
