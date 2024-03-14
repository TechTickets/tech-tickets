#![feature(associated_type_defaults)]
#![allow(async_fn_in_trait)]

#[cfg(all(feature = "client"))]
pub mod client;

#[cfg(all(feature = "server", not(feature = "client")))]
pub mod routes;

#[cfg(all(feature = "client"))]
pub mod routes;
