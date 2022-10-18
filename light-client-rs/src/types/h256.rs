use alloc::string::String;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct H256(pub [u8; 32]);

impl From<[u8; 32]> for H256 {
    fn from(bytes: [u8; 32]) -> Self {
        H256(bytes)
    }
}

impl From<H256> for [u8; 32] {
    fn from(h: H256) -> Self {
        h.0
    }
}

impl H256 {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err("Invalid length for H256".into());
        }
        let mut h = H256::default();
        h.0.copy_from_slice(bytes);
        Ok(h)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
