#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! I/O-free coroutines for the Microsoft Graph API.
//!
//! Each module mirrors a Microsoft Graph resource; see the reference at
//! <https://learn.microsoft.com/en-us/graph/api/overview>.

extern crate alloc;
#[cfg(feature = "client")]
extern crate std;

pub mod coroutine;
pub mod v1;
