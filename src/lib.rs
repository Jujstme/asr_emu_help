#![no_std]
pub mod shared;

#[cfg(feature = "ps1")]
pub mod ps1;

#[cfg(feature = "genesis")]
pub mod genesis;

#[cfg(feature = "wii")]
pub mod wii;