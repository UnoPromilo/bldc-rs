pub const FLASH_PAGE_SIZE: u32 = 2048;

unsafe extern "C" {
    static __settings_start: u32;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActivePage {
    A,
    B,
}

impl ActivePage {

    #[cfg(feature = "hardware-support")]
    pub fn start(&self) -> u32 {
        match self {
            ActivePage::A => unsafe { __settings_start },
            ActivePage::B => unsafe { __settings_start + FLASH_PAGE_SIZE },
        }
    }

    #[cfg(not(feature = "hardware-support"))]
    pub fn start(&self) -> u32 {
        0
    }

    pub fn other(&self) -> Self {
        match self {
            ActivePage::A => ActivePage::B,
            ActivePage::B => ActivePage::A,
        }
    }
}
