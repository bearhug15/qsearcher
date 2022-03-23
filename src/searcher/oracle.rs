use qip::{Register, OpBuilder};
use crate::searcher::utils::LayeredRegister;

pub trait Oracle{
    //fn init(service_data: Vec<u8>)->Self;
    fn set_service_data(&mut self, service_data: Option<Vec<u8>>);
    //fn get_builder<'a>(&mut self)->&'a mut OpBuilder;
    fn get_main_qubit_usage(&self)->Option<usize>;
    fn make_prediction(&self, main_data: LayeredRegister)->(LayeredRegister, Register);
    fn init_main_data(&mut self,builder: &mut OpBuilder, data: &Vec<u8>, nonce_size_bits: u64, main_qubit_usage:usize) -> LayeredRegister;
}