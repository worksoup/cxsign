use crate::sign::{RawSign, SignTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct NormalSign {
    pub(crate) raw_sign: RawSign,
}

impl SignTrait for NormalSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
