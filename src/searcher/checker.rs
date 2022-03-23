pub trait Checker{
    fn check(&self, result: Vec<u8>)->bool;
}