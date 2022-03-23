use qip::{Register, OpBuilder, UnitaryBuilder, CircuitError};
use std::rc::Rc;
use qip::program;

pub struct LayeredRegister {
    //prefix:Vec<Register>,
    layers: Vec<Register>,
    //postfix: Vec<Register>,
    current_layer_index: usize,
    start_depth:usize,
    width:u64,
    dif_range: Option<(u64,u64)>
}

impl LayeredRegister {
    pub fn new(builder: &mut OpBuilder, width: u64, depth: usize) -> Self {
        let mut layers: Vec<Register> = Vec::with_capacity(depth);
        //let mut prefix: Vec<Register> = Vec::with_capacity(depth);
        //let mut postfix: Vec<Register> = Vec::with_capacity(depth);
        let current_layer_index: usize = 0;
        let start_depth: usize = depth;
        for _ in 0..depth {
            let reg = match builder.register(width) {
                Ok(res) => res,
                Err(err) => {
                    panic!("{}",err.to_string())
                }
            };
            layers.push(reg);
        }
        LayeredRegister { layers, current_layer_index, start_depth,width, dif_range: None }
    }
    pub fn start_depth(&self) -> usize {
        self.start_depth
    }
    pub fn depth(&self) -> usize {
        self.start_depth - self.current_layer_index
    }
    pub fn width(&self) -> u64 {self.width}
    pub fn pop_layer(&mut self) -> Register {
        if self.current_layer_index==self.start_depth {
            panic!("LayeredRegister depleted")
        } else{
            self.current_layer_index = self.current_layer_index+1;
            self.layers.pop().unwrap()
        }
    }
    pub fn set_dif_range(&mut self, start:u64,end:u64){
        self.dif_range = Some((start,end));
    }
    pub fn get_dif_range(&self) -> Option<(u64, u64)> {
        self.dif_range
    }
    /*pub fn apply_to_layers1(self, mut f: Box<dyn FnMut(&mut dyn UnitaryBuilder, Register, Option<(u64, u64)>) -> Register>, builder:&mut OpBuilder) -> Self
    {
        let  current_layer_index =self.current_layer_index;
        let start_depth = self.start_depth;
        let width = self.width;
        let dif_range = self.dif_range;
        let layers: Vec<Register> = self.layers.into_iter().map(|reg|{
            f(builder,reg,dif_range)
        }).collect();
        LayeredRegister{layers,current_layer_index, start_depth,width, dif_range }
        //unimplemented!()
    }*/

    pub fn apply_to_layers1_in_range(mut self, mut f: Box<dyn Fn(&mut dyn UnitaryBuilder, Register) -> Register>, builder:&mut OpBuilder,start:u64,end:u64) -> Self{
        let current_layer_index =self.current_layer_index;
        let start_depth = self.start_depth;
        let width = self.width;
        let dif_range = self.dif_range;
        let layers: Vec<Register> = Vec::with_capacity(self.layers.len());
        //let mut func = func_converter1(f);
        let func = (move |builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>| -> Result<Vec<Register>, CircuitError>{
            let reg = regs.pop().unwrap();
            Ok(vec![f(builder,reg)])
        });
        let layers: Vec<Register> = self.layers.into_iter().map(|reg|{
            program!(builder,reg;
            func reg[start..end];
            ).unwrap()
        }).collect();
        LayeredRegister{layers,current_layer_index, start_depth,width, dif_range }
    }
    /*pub fn apply_to_sliced_layers1(self, mut f: Box<FnMut(&mut OpBuilder, Register) -> Register>,builder:&mut OpBuilder,start_index:u64,end_index:u64)  -> Self {
        let  current_layer_index =self.current_layer_index;
        let start_len = self.start_len;

        let layers: Vec<Register> = self.layers.into_iter().map(|reg|{
            program!(&mut builder,reg;
            f reg[start_index..end_index];
            )?;
        }).collect();
        LayeredRegister{layers,current_layer_index,start_len}
    }*/
}
fn func_converter1(mut f: Box<dyn FnMut(&mut dyn UnitaryBuilder, Register) -> Register>) ->Box<dyn FnMut(&mut dyn UnitaryBuilder, Vec<Register>) -> Result<Vec<Register>, CircuitError>>{
    /*fn func(builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
        let reg = regs.pop().unwrap();
        f(builder,reg)
    }*/
    Box::new(move |builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>| -> Result<Vec<Register>, CircuitError>{
        let reg = regs.pop().unwrap();
        Ok(vec![f(builder,reg)])
    })
}