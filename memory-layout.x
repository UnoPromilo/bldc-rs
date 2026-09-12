/* STM32G474RE flash layout in dual-bank mode. */
FLASH_BASE = 0x08000000;
FLASH_SIZE = 512K;
FLASH_BANK_SIZE = 256K;
FLASH_ERASE_SIZE = 2K;
FLASH_WRITE_SIZE = 8;

BANK1_ORIGIN = FLASH_BASE;
BANK2_ORIGIN = FLASH_BASE + FLASH_BANK_SIZE;

BOOTLOADER_ORIGIN = BANK1_ORIGIN;
BOOTLOADER_SIZE = 48K;

BOOTLOADER_STATE_ORIGIN = BOOTLOADER_ORIGIN + BOOTLOADER_SIZE;
BOOTLOADER_STATE_SIZE = 8K;

ACTIVE_ORIGIN = BOOTLOADER_STATE_ORIGIN + BOOTLOADER_STATE_SIZE;
ACTIVE_SIZE = 196K;

SETTINGS_ORIGIN = ACTIVE_ORIGIN + ACTIVE_SIZE;
SETTINGS_SIZE = 4K;

DFU_ORIGIN = BANK2_ORIGIN;
DFU_SIZE = FLASH_BANK_SIZE;

RAM_ORIGIN = 0x20000000;
RAM_SIZE = 128K;

ACTIVE_PAGE_COUNT = ACTIVE_SIZE / FLASH_ERASE_SIZE;
BOOTLOADER_STATE_REQUIRED_SIZE =
    FLASH_WRITE_SIZE * (2 + 4 * ACTIVE_PAGE_COUNT);

ASSERT(FLASH_SIZE == 2 * FLASH_BANK_SIZE,
       "Flash must contain two equal banks");
ASSERT((BOOTLOADER_ORIGIN % FLASH_ERASE_SIZE) == 0,
       "Bootloader origin must be erase-page aligned");
ASSERT((BOOTLOADER_STATE_ORIGIN % FLASH_ERASE_SIZE) == 0,
       "Bootloader state origin must be erase-page aligned");
ASSERT((ACTIVE_ORIGIN % FLASH_ERASE_SIZE) == 0,
       "Active firmware origin must be erase-page aligned");
ASSERT((SETTINGS_ORIGIN % FLASH_ERASE_SIZE) == 0,
       "Settings origin must be erase-page aligned");
ASSERT((DFU_ORIGIN % FLASH_ERASE_SIZE) == 0,
       "DFU origin must be erase-page aligned");
ASSERT((BOOTLOADER_SIZE % FLASH_ERASE_SIZE) == 0,
       "Bootloader size must be erase-page aligned");
ASSERT((BOOTLOADER_STATE_SIZE % FLASH_ERASE_SIZE) == 0,
       "Bootloader state size must be erase-page aligned");
ASSERT((ACTIVE_SIZE % FLASH_ERASE_SIZE) == 0,
       "Active firmware size must be erase-page aligned");
ASSERT((SETTINGS_SIZE % FLASH_ERASE_SIZE) == 0,
       "Settings size must be erase-page aligned");
ASSERT((DFU_SIZE % FLASH_ERASE_SIZE) == 0,
       "DFU size must be erase-page aligned");
ASSERT(BOOTLOADER_STATE_ORIGIN == BOOTLOADER_ORIGIN + BOOTLOADER_SIZE,
       "Bootloader and state partitions must be contiguous");
ASSERT(ACTIVE_ORIGIN == BOOTLOADER_STATE_ORIGIN + BOOTLOADER_STATE_SIZE,
       "State and active partitions must be contiguous");
ASSERT(SETTINGS_ORIGIN == ACTIVE_ORIGIN + ACTIVE_SIZE,
       "Active firmware and settings partitions must not overlap");
ASSERT(SETTINGS_ORIGIN + SETTINGS_SIZE == BANK2_ORIGIN,
       "Bank 1 partitions must end at the Bank 2 boundary");
ASSERT(DFU_ORIGIN == BANK2_ORIGIN,
       "DFU must start at the Bank 2 origin");
ASSERT(DFU_ORIGIN + DFU_SIZE == FLASH_BASE + FLASH_SIZE,
       "DFU must fit exactly in Bank 2");
ASSERT(DFU_SIZE >= ACTIVE_SIZE + FLASH_ERASE_SIZE,
       "Embassy swap requires DFU to exceed ACTIVE by one erase page");
ASSERT(BOOTLOADER_STATE_SIZE >= BOOTLOADER_STATE_REQUIRED_SIZE,
       "Bootloader state partition is too small for swap progress");
ASSERT(SETTINGS_SIZE == 2 * FLASH_ERASE_SIZE,
       "Settings storage requires two erase pages");
