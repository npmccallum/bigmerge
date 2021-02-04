// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::all)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use koine::{Backend, Contract};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keep {
    pub uuid: Uuid,
    pub contract: Contract,
}
