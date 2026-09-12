INCLUDE memory-layout.x

MEMORY
{
    FLASH                             : ORIGIN = BOOTLOADER_ORIGIN, LENGTH = BOOTLOADER_SIZE
    BOOTLOADER_STATE                  : ORIGIN = BOOTLOADER_STATE_ORIGIN, LENGTH = BOOTLOADER_STATE_SIZE
    ACTIVE                            : ORIGIN = ACTIVE_ORIGIN, LENGTH = ACTIVE_SIZE
    SETTINGS                          : ORIGIN = SETTINGS_ORIGIN, LENGTH = SETTINGS_SIZE
    DFU                               : ORIGIN = DFU_ORIGIN, LENGTH = DFU_SIZE
    RAM   (rwx)                       : ORIGIN = RAM_ORIGIN, LENGTH = RAM_SIZE
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - BANK1_ORIGIN;
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - BANK1_ORIGIN;

__bootloader_active_start = ORIGIN(ACTIVE) - BANK1_ORIGIN;
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - BANK1_ORIGIN;

__bootloader_dfu_start = ORIGIN(DFU) - BANK2_ORIGIN;
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - BANK2_ORIGIN;