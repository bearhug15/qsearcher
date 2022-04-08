use crate::searcher::checker::Checker;
use sha2::{Sha256,Digest};

pub struct SHA256Checker{
    data: Vec<u8>,
    limit: [u8;32]
}

impl SHA256Checker {
    pub fn init(data:Vec<u8>,limit:[u8;32])->SHA256Checker{
        SHA256Checker{data,limit}
    }
}

impl Checker for SHA256Checker{
    fn check_nonce(&self, mut result: Vec<u8>) -> bool {
        let mut  data = self.data.clone();
        data.append(&mut result);
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash: Vec<u8> = hasher.finalize().into_iter().collect();
        hash<Vec::from(self.limit)
    }

}