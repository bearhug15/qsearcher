pub trait Checker{
    fn check_nonce(&self, result: Vec<u8>)->bool;
}