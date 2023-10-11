#![doc = include_str!("../README.md")]

pub use client::{ClientCredential, ConfidentialClient};
pub use error::Error;

mod authority;
mod client;
mod error;
