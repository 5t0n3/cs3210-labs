#![no_std]
#![feature(core_intrinsics)]
#![feature(decl_macro)]
#![feature(never_type)]
// bilge requirements
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_mut_refs)]

pub mod atags;
pub mod common;
pub mod gpio;
pub mod sd;
pub mod timer;
pub mod uart;
