// SPDX-License-Identifier: Apache-2.0

use super::{Command, Error};

use ciborium::de::from_reader;
use koine::Contract;
use reqwest::header::CONTENT_TYPE;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(StructOpt)]
pub struct List {
    /// The server base URL
    #[structopt(short, long, env = "ENARX_SERVER")]
    url: reqwest::Url,
}

#[async_trait::async_trait]
impl Command for List {
    async fn run(self) -> Result<(), Error> {
        let url = self.url.join("contracts")?;
        let response = reqwest::get(url).await?;
        let response = response.error_for_status()?;
        let response = Error::check_header(response, CONTENT_TYPE, "application/cbor")?;

        let contracts: Vec<Contract> = response.decode(|bytes| from_reader(bytes)).await?;
        for contract in contracts {
            println!("{} ({})", contract.uuid, contract.backend.as_str());
        }

        Ok(())
    }
}

#[derive(StructOpt)]
pub struct Show {
    /// The server base URL
    #[structopt(short, long, env = "ENARX_SERVER")]
    url: reqwest::Url,

    /// The contract UUID
    uuid: Uuid,
}

#[async_trait::async_trait]
impl Command for Show {
    async fn run(self) -> Result<(), Error> {
        let uuid = self.uuid.to_hyphenated().to_string();
        let url = self.url.join("contracts/")?.join(&uuid)?;
        let response = reqwest::get(url).await?;
        let response = response.error_for_status()?;
        let response = Error::check_header(response, CONTENT_TYPE, "application/cbor")?;

        let contract: Contract = response.decode(|bytes| from_reader(bytes)).await?;
        println!("{:#?}", contract);
        Ok(())
    }
}

#[derive(StructOpt)]
pub enum Contracts {
    List(List),
    Show(Show),
}

#[async_trait::async_trait]
impl Command for Contracts {
    async fn run(self) -> Result<(), Error> {
        match self {
            Self::List(cmd) => cmd.run().await,
            Self::Show(cmd) => cmd.run().await,
        }
    }
}
