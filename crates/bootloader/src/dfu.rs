use core::sync::atomic::{AtomicBool, Ordering};
use defmt::info;
use embassy_boot_stm32::{AlignedBuffer, BlockingFirmwareUpdater};
use embassy_usb::class::dfu::consts::{DfuAttributes, Status};
use embassy_usb::class::dfu::dfu_mode::Handler;
use embassy_usb::control::{InResponse, OutResponse, Request};
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, FunctionBuilder};
use embassy_usb_dfu::dfu::UsbDfuState;
use embassy_usb_dfu::{Reset, ResetImmediate};
use embedded_storage::nor_flash::NorFlash;
use hardware::BoardLeds;

use crate::image_size::{DownloadError, DownloadTracker};
// This code is derived from embassy-usb and embassy-usb-dfu.
// Modifications were made to ensure that restarts triggered by dfu-util function correctly.

pub(crate) const USB_CLASS_APPN_SPEC: u8 = 0xFE;
pub(crate) const APPN_SPEC_SUBCLASS_DFU: u8 = 0x01;
pub(crate) const DFU_PROTOCOL_DFU: u8 = 0x02;
pub(crate) const DESC_DFU_FUNCTIONAL: u8 = 0x21;

// Shared by the USB protocol wrapper and firmware writer to poison a rejected transfer.
static DOWNLOAD_FAILED: AtomicBool = AtomicBool::new(false);
static DOWNLOAD_FINISHED: AtomicBool = AtomicBool::new(false);

pub(crate) struct FirmwareHandler<'d, DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize> {
    updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
    offset: usize,
    buffer: AlignedBuffer<BLOCK_SIZE>,
    download: DownloadTracker<BLOCK_SIZE>,
}

impl<'d, DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize>
    FirmwareHandler<'d, DFU, STATE, BLOCK_SIZE>
{
    fn new(updater: BlockingFirmwareUpdater<'d, DFU, STATE>, maximum_size: usize) -> Self {
        Self {
            updater,
            offset: 0,
            buffer: AlignedBuffer([0; BLOCK_SIZE]),
            download: DownloadTracker::new(maximum_size),
        }
    }
}

impl<DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize> Handler
    for FirmwareHandler<'_, DFU, STATE, BLOCK_SIZE>
{
    fn start(&mut self) -> Result<(), Status> {
        self.offset = 0;
        self.download.reset();
        DOWNLOAD_FAILED.store(false, Ordering::Relaxed);
        DOWNLOAD_FINISHED.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<(), Status> {
        let pending = match self.download.check_chunk(data.len()) {
            Ok(pending) => pending,
            Err(error) => {
                DOWNLOAD_FAILED.store(true, Ordering::Relaxed);
                return Err(match error {
                    DownloadError::EmptyChunk
                    | DownloadError::ChunkTooLarge
                    | DownloadError::ImageTooLarge
                    | DownloadError::DataAfterShortBlock
                    | DownloadError::Failed => Status::ErrAddress,
                    DownloadError::ImageTooSmall => Status::ErrFile,
                });
            }
        };

        self.buffer.0.fill(0xFF);
        self.buffer.0[..data.len()].copy_from_slice(data);
        if self
            .updater
            .write_firmware(self.offset, &self.buffer.0)
            .is_err()
        {
            self.download.fail();
            DOWNLOAD_FAILED.store(true, Ordering::Relaxed);
            return Err(Status::ErrWrite);
        }

        self.offset += data.len();
        self.download.commit(pending);
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Status> {
        if DOWNLOAD_FAILED.load(Ordering::Relaxed) {
            return Err(Status::ErrNotDone);
        }
        self.download.finish().map_err(|_| Status::ErrFile)?;

        match self.updater.mark_updated() {
            Ok(()) => {
                DOWNLOAD_FINISHED.store(true, Ordering::Relaxed);
                Ok(())
            }
            Err(_) => {
                self.download.fail();
                DOWNLOAD_FAILED.store(true, Ordering::Relaxed);
                Err(Status::ErrUnknown)
            }
        }
    }

    fn system_reset(&mut self) {
        ResetImmediate.sys_reset()
    }
}

pub fn new_state<'a, DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize>(
    updater: BlockingFirmwareUpdater<'a, DFU, STATE>,
    board_leds: BoardLeds<'a>,
    maximum_size: usize,
) -> DfuState<'a, FirmwareHandler<'a, DFU, STATE, BLOCK_SIZE>> {
    let handler = FirmwareHandler::new(updater, maximum_size);
    DfuState::new(handler, board_leds)
}

pub struct DfuState<'a, H: Handler> {
    inner: UsbDfuState<H>,
    attrs: DfuAttributes,
    board_leds: BoardLeds<'a>,
}

impl<'a, H: Handler> DfuState<'a, H> {
    pub fn new(handler: H, board_leds: BoardLeds<'a>) -> Self {
        let attrs = DfuAttributes::CAN_DOWNLOAD | DfuAttributes::MANIFESTATION_TOLERANT;
        let inner = UsbDfuState::new(handler, attrs);
        Self {
            inner,
            attrs,
            board_leds,
        }
    }
}

impl<H: Handler> embassy_usb::Handler for DfuState<'_, H> {
    fn reset(&mut self) {
        self.inner.reset();
        if DOWNLOAD_FINISHED.load(Ordering::Relaxed) {
            info!("Goodbye!");
            ResetImmediate.sys_reset();
        }
    }

    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        let is_download = req.request == 1;
        if is_download {
            self.board_leds.red.toggle();
        }

        let response = self.inner.control_out(req, data);
        if matches!(response, Some(OutResponse::Rejected)) {
            DOWNLOAD_FAILED.store(true, Ordering::Relaxed);
        }

        response
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        self.inner.control_in(req, buf)
    }
}

pub fn usb_dfu<'d, D: Driver<'d>, DFU: NorFlash, STATE: NorFlash, const BLOCK_SIZE: usize>(
    builder: &mut Builder<'d, D>,
    state: &'d mut DfuState<FirmwareHandler<DFU, STATE, BLOCK_SIZE>>,
    func_modifier: impl Fn(&mut FunctionBuilder<'_, 'd, D>),
) {
    let mut func = builder.function(0x00, 0x00, 0x00);

    func_modifier(&mut func);

    let mut iface = func.interface();
    let mut alt = iface.alt_setting(
        USB_CLASS_APPN_SPEC,
        APPN_SPEC_SUBCLASS_DFU,
        DFU_PROTOCOL_DFU,
        None,
    );
    alt.descriptor(
        DESC_DFU_FUNCTIONAL,
        &[
            state.attrs.bits(),
            0xc4,
            0x09,
            (BLOCK_SIZE & 0xff) as u8,
            ((BLOCK_SIZE & 0xff00) >> 8) as u8,
            0x10,
            0x01,
        ],
    );

    drop(func);
    builder.handler(state);
}
