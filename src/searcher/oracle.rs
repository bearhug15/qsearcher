use qip::{OpBuilder, Register};
use qip::pipeline::RegisterInitialState;
use crate::searcher::utils::LayeredRegister;

pub trait Oracle{
    fn set_service_data(&mut self, service_data: Option<Vec<u8>>);
    fn get_main_qubit_usage(&self)->Option<usize>;
    fn make_prediction(&mut self, builder: &mut OpBuilder,main_data: LayeredRegister)->(LayeredRegister, Register, Vec<RegisterInitialState<f32>>);
    fn init_main_data(&mut self,builder: &mut OpBuilder, data: &Vec<u8>, nonce_size_bits: u64, main_qubit_usage:usize) -> (LayeredRegister,Vec<RegisterInitialState<f32>>);
}