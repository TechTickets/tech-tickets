#![feature(associated_type_defaults)]
#![allow(async_fn_in_trait)]

pub use http;

#[cfg(feature = "client")]
pub mod client;

pub mod routes;
