#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod errors;
pub mod instructions;
pub mod states;
pub mod constants;

pinocchio_pubkey::declare_id!("E4U89BDRNy7Z6ZFaHPKz1VG8qk384jWv7Cgacp8F8x7X");