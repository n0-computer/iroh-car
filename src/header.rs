use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// A car header.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CarHeader {
    V1(CarHeaderV1),
}

impl CarHeader {
    pub fn new_v1(roots: Vec<Cid>) -> Self {
        Self::V1(roots.into())
    }

    pub fn decode(buffer: &[u8]) -> Result<Self, Error> {
        let header: CarHeaderV1 =
            serde_ipld_dagcbor::from_reader(buffer).map_err(|e| Error::Parsing(e.to_string()))?;

        if header.roots.is_empty() {
            return Err(Error::Parsing("empty CAR file".to_owned()));
        }

        if header.version != 1 {
            return Err(Error::InvalidFile(
                "Only CAR file version 1 is supported".to_string(),
            ));
        }

        Ok(CarHeader::V1(header))
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        match self {
            CarHeader::V1(ref header) => {
                let res = serde_ipld_dagcbor::to_vec(header).expect("vec");
                Ok(res)
            }
        }
    }

    pub fn roots(&self) -> &[Cid] {
        match self {
            CarHeader::V1(header) => &header.roots,
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            CarHeader::V1(_) => 1,
        }
    }
}

/// CAR file header version 1.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CarHeaderV1 {
    pub roots: Vec<Cid>,
    pub version: u64,
}

impl CarHeaderV1 {
    /// Creates a new CAR file header
    pub fn new(roots: Vec<Cid>, version: u64) -> Self {
        Self { roots, version }
    }
}

impl From<Vec<Cid>> for CarHeaderV1 {
    fn from(roots: Vec<Cid>) -> Self {
        Self { roots, version: 1 }
    }
}

#[cfg(test)]
mod tests {
    use multihash_codetable::MultihashDigest;

    use super::*;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn symmetric_header_v1() {
        let digest = multihash_codetable::Code::Blake3_256.digest(b"test");
        let cid = Cid::new_v1(0x71, digest);

        let header = CarHeaderV1::from(vec![cid]);

        let bytes = serde_ipld_dagcbor::to_vec(&header).unwrap();

        assert_eq!(
            serde_ipld_dagcbor::from_slice::<CarHeaderV1>(&bytes).unwrap(),
            header
        );
    }
}
