// SPDX-License-Identifier: Apache-2.0

use super::backend::Backend;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Contract {
    pub uuid: Uuid,
    pub backend: Backend,
}
