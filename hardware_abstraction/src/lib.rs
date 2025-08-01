#![no_std]
#![allow(async_fn_in_trait)]

pub mod angle_sensor;
pub mod models;
pub mod motor_driver;
pub mod current_sensor;

#[cfg(test)]
extern crate alloc;
