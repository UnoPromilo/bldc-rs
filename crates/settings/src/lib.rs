#![no_std]

#[cfg(test)]
extern crate alloc;

pub mod flash;
pub mod format;
pub mod manager;
pub  mod state;
mod scanner;
mod helpers;
mod page;
mod errors;
#[cfg(feature = "hardware-support")]
pub mod hardware;
pub  mod storage;