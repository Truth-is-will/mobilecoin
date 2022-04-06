// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Ledger Streaming API.

#![feature(assert_matches)]
#![feature(type_alias_impl_trait)]
#![deny(missing_docs)]

mod autogenerated_code {
    use mc_api::{blockchain, external};

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}
mod components;
mod convert;
mod error;
mod response;
mod traits;

#[cfg(any(test, feature = "test_utils"))]
pub mod test_utils;

pub use self::{
    autogenerated_code::*,
    components::BlockStreamComponents,
    convert::*,
    error::{Error, Result},
    response::{make_subscribe_response, parse_subscribe_response},
    traits::{BlockFetcher, BlockStream},
};