// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Nil,
    Sev,
    Sgx,
    Kvm,
}

#[derive(Copy, Clone, Debug)]
pub struct UnknownBackend;

impl std::str::FromStr for Backend {
    type Err = UnknownBackend;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "nil" => Ok(Self::Nil),
            "sev" => Ok(Self::Sev),
            "sgx" => Ok(Self::Sgx),
            "kvm" => Ok(Self::Kvm),
            _ => Err(UnknownBackend),
        }
    }
}

impl ToString for Backend {
    #[inline]
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Backend {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Backend::Nil => "nil",
            Backend::Sev => "sev",
            Backend::Sgx => "sgx",
            Backend::Kvm => "kvm",
        }
    }
}
