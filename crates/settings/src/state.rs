use core::sync::atomic::AtomicU32;

pub struct Settings {
    pub current_limit: AtomicU32,
    pub voltage_limit: AtomicU32,
}