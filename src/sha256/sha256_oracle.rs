use crate::searcher::oracle::Oracle;
use qip::{OpBuilder, Register};
use crate::searcher::utils::LayeredRegister;

pub struct SHA256Oracle{
    //builder: OpBuilder,
    service_data:Vec<u8>
}

impl Oracle for SHA256Oracle{
    fn set_service_data(&mut self, service_data: Option<Vec<u8>>) {
        unimplemented!()
    }

    /*fn get_builder<'a>(&mut self) -> &mut OpBuilder {
        &mut self.builder
    }*/

    fn get_main_qubit_usage(&self) -> Option<usize> {
        None
    }

    fn make_prediction(&self, main_data: LayeredRegister) -> (LayeredRegister, Register) {
        unimplemented!()
    }


    fn init_main_data(&mut self,builder: &mut OpBuilder, data: &Vec<u8>, nonce_size_bits: u64, main_qubit_usage:usize) -> LayeredRegister {
        let main_size_bits = data.len() as u64 *8 + nonce_size_bits;
        let full_size_bits = ((main_size_bits+1+64)%512 +main_size_bits+1+64) as u64;
        LayeredRegister::new(builder,full_size_bits,main_qubit_usage);
        unimplemented!()
    }
}