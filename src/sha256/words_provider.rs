use std::collections::HashMap;
use itertools::Itertools;
use qip::{OpBuilder, Register};
use qip::pipeline::RegisterInitialState;

use crate::searcher::utils::LayeredRegister;

pub(crate) struct WordsProvider {
    //region->word_number->words
    words: HashMap<u64,HashMap<u64,Vec<Register>>>
}

impl WordsProvider {
    pub fn init()->Self{
        WordsProvider{ words: HashMap::new() }
    }
    pub fn get_word(&mut self, builder:&mut OpBuilder, regs: &mut LayeredRegister, region:u64,i:u64) -> (Register,Vec<RegisterInitialState<f32>>){
        let region_result = self.words.get_mut(&region);
        match region_result {
            Some(region)=>{

            }
            None =>{
                let (regs,init) = parse_data_into_chunks(builder,regs.pop_layer());
                let regs_chunks = regs.into_iter().chunks(16)
                for (chunk,region) in regs.into_iter().chunks(16)
                for region in 0..regs.len()/16{
                    let mut map = HashMap::<u64,Register>::new();

                }
            }
        };
        unimplemented!()
    }
    pub fn return_word(&mut self,word:Register, region:u64,i:u64){
        unimplemented!()
    }

}
fn parse_data_into_chunks(builder:&mut OpBuilder, reg:Register)->(Vec<Register>, Vec<RegisterInitialState<f32>>){
    unimplemented!()
}
fn get_word_from_region(builder:&mut OpBuilder, map: &mut HashMap<u64,Register>, i: u64)->(Register,Vec<RegisterInitialState<f32>>){
    unimplemented!()
}