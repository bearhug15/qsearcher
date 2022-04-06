use qip::{OpBuilder, Register};
use qip::pipeline::RegisterInitialState;
use crate::searcher::utils::LayeredRegister;

pub(crate) struct WordsProvider {

}

impl WordsProvider {
    pub fn init()->Self{
        unimplemented!()
    }
    pub fn get_word(&mut self, builder:&mut OpBuilder, regs: &mut LayeredRegister, region:u64,i:u64) -> (Register,Vec<RegisterInitialState<f32>>){
        unimplemented!()
    }
    pub fn return_word(&mut self,word:Register, region:u64,i:u64){

    }
}