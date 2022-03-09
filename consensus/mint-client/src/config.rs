// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Command line configuration for the consensus mint client.

use mc_util_uri::ConsensusClientUri;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mc-consensus-mint-client",
    about = "MobileCoin Consensus Mint Client"
)]
pub struct Config {
    /// URI of consensus node to connect to.
    #[structopt(long)]
    pub node: ConsensusClientUri,
}