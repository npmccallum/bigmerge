// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::all)]
#![allow(clippy::redundant_closure)]

mod contracts;
mod error;

use error::Error;

use structopt::StructOpt;

#[async_trait::async_trait]
trait Command: StructOpt {
    async fn run(self) -> Result<(), Error>;
}

#[derive(StructOpt)]
pub enum Commands {
    Contracts(contracts::Contracts),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    match Commands::from_args() {
        Commands::Contracts(cmd) => cmd.run().await,
    }
}
