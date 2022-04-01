use qip::{Register, OpBuilder, UnitaryBuilder, CircuitError};
use std::rc::Rc;
use qip::program;
use qip::pipeline::RegisterInitialState;

pub struct LayeredRegister {
    //prefix:Vec<Register>,
    layers: Vec<Register>,
    //postfix: Vec<Register>,
    current_layer_index: usize,
    start_depth: usize,
    width: u64,
    dif_range: Option<(u64, u64)>,
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
                    panic!("{}", err.to_string())
                }
            };
            layers.push(reg);
        }
        LayeredRegister { layers, current_layer_index, start_depth, width, dif_range: None }
    }
    pub fn new_initialized(builder: &mut OpBuilder, width: u64, depth: usize, data: &Vec<u8>) -> (Self, Vec<RegisterInitialState<f32>>) {
        if width != (data.len() * 8) as u64 {
            panic!("Not enough data to initialize")
        }
        let mut layers: Vec<Register> = Vec::with_capacity(depth);
        let current_layer_index: usize = 0;
        let start_depth: usize = depth;
        let mut full_init = Vec::<RegisterInitialState<f32>>::new();
        for _ in 0..depth {
            let (comp_data, mut init) = data.iter().map(|val| {
                let qubits = builder.register(8).unwrap();
                let init: RegisterInitialState<f32> = qubits.handle().make_init_from_index(u64::from(val.clone())).unwrap();
                (qubits, init)
            }).fold((Vec::<Register>::new(), Vec::<RegisterInitialState<f32>>::new()), |(mut reg_collector, mut vec), (mut reg, init)| {
                reg_collector.append(&mut vec![reg]);
                vec.append(&mut vec![init]);
                (reg_collector, vec)
            });
            let pre_merged = builder.merge(comp_data).unwrap();
            layers.push(pre_merged);
            full_init.append(&mut init);
        }
        (LayeredRegister { layers, current_layer_index, start_depth, width, dif_range: None }, full_init)
    }
    pub fn start_depth(&self) -> usize {
        self.start_depth
    }
    pub fn depth(&self) -> usize {
        self.start_depth - self.current_layer_index
    }
    pub fn width(&self) -> u64 { self.width }
    pub fn pop_layer(&mut self) -> Register {
        if self.current_layer_index == self.start_depth {
            panic!("LayeredRegister depleted")
        } else {
            self.current_layer_index = self.current_layer_index + 1;
            self.layers.pop().unwrap()
        }
    }
    pub fn set_dif_range(&mut self, start: u64, end: u64) {
        self.dif_range = Some((start, end));
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

    pub fn apply_to_layers1_in_range(mut self, mut f: Box<dyn Fn(&mut dyn UnitaryBuilder, Register) -> Register>, builder: &mut OpBuilder, start: u64, end: u64) -> Self {
        let current_layer_index = self.current_layer_index;
        let start_depth = self.start_depth;
        let width = self.width;
        let dif_range = self.dif_range;
        let layers: Vec<Register> = Vec::with_capacity(self.layers.len());
        //let mut func = func_converter1(f);
        let func = (move |builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>| -> Result<Vec<Register>, CircuitError>{
            let reg = regs.pop().unwrap();
            Ok(vec![f(builder, reg)])
        });
        let layers: Vec<Register> = self.layers.into_iter().map(|reg| {
            program!(builder,reg;
            func reg[start..end];
            ).unwrap()
        }).collect();
        LayeredRegister { layers, current_layer_index, start_depth, width, dif_range }
    }
}

fn func_converter1(mut f: Box<dyn FnMut(&mut dyn UnitaryBuilder, Register) -> Register>) -> Box<dyn FnMut(&mut dyn UnitaryBuilder, Vec<Register>) -> Result<Vec<Register>, CircuitError>> {
    /*fn func(builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
        let reg = regs.pop().unwrap();
        f(builder,reg)
    }*/
    Box::new(move |builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>| -> Result<Vec<Register>, CircuitError>{
        let reg = regs.pop().unwrap();
        Ok(vec![f(builder, reg)])
    })
}

pub fn register_rotr(builder: &mut OpBuilder, reg: Register, mut step: u64) -> Register {
    let n = reg.n();
    step = step % n;

    unimplemented!()
}

pub fn register_and(builder: &mut OpBuilder, mut reg1: Register, mut reg2: Register, mut step: u64) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg1.n();
    let (mut res, mut init) = zeroed_register(builder, n);
    for i in 0..n {
        let (buff1, buff2, res_buff) = program!(builder, reg1, reg2,res;
            ccnot_wrapper reg1[i],reg2[i],res[i];
        ).unwrap();
        reg1 = buff1;
        reg2 = buff2;
        res = res_buff;
    }
    (reg1,reg2,res,init)
}
pub fn register_or(builder: &mut OpBuilder, mut reg1: Register, mut reg2: Register, mut step: u64) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg1.n();
    let (mut res, mut init) = oneed_register(builder, n);
    reg1 = builder.not(reg1);
    reg2 = builder.not(reg2);
    for i in 0..n {
        let (buff1, buff2, res_buff) = program!(builder, reg1, reg2,res;
            ccnot_wrapper reg1[i],reg2[i],res[i];
        ).unwrap();
        reg1 = buff1;
        reg2 = buff2;
        res = res_buff;
    }
    reg1 = builder.not(reg1);
    reg2 = builder.not(reg2);
    (reg1,reg2,res,init)

}


fn ccnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let r3 = rs.pop().unwrap();
    let r2 = rs.pop().unwrap();
    let r1 = rs.pop().unwrap();
    let (r1, r2, r3) = b.ccnot(r1, r2, r3);
    Ok(vec![r1, r2, r3])
}

fn zeroed_register(builder: &mut OpBuilder, len: u64) -> (Register, Vec<RegisterInitialState<f32>>) {
    let chunks_amount = len / 8;
    let remains_len = len % 8;
    let buff = chunks_amount + if remains_len != 0 { 1 } else { 0 };
    let mut chunks = Vec::<Register>::with_capacity(buff as usize);
    let mut inits = Vec::<RegisterInitialState<f32>>::with_capacity(buff as usize);
    for _ in 0..chunks_amount {
        let reg = builder.register(8).unwrap();
        let init = reg.handle().make_init_from_index(0).unwrap();
        chunks.push(reg);
        inits.push(init)
    }
    if remains_len != 0 {
        let reg = builder.register(remains_len).unwrap();
        let init = reg.handle().make_init_from_index(0).unwrap();
        chunks.push(reg);
        inits.push(init)
    }
    let reg = builder.merge(chunks).unwrap();
    (reg, inits)
}
fn oneed_register(builder: &mut OpBuilder, len: u64) -> (Register, Vec<RegisterInitialState<f32>>) {
    let chunks_amount = len / 8;
    let remains_len = len % 8;
    let buff = chunks_amount + if remains_len != 0 { 1 } else { 0 };
    let mut chunks = Vec::<Register>::with_capacity(buff as usize);
    let mut inits = Vec::<RegisterInitialState<f32>>::with_capacity(buff as usize);
    for _ in 0..chunks_amount {
        let reg = builder.register(8).unwrap();
        let init = reg.handle().make_init_from_index(0b11111111).unwrap();
        chunks.push(reg);
        inits.push(init)
    }
    if remains_len != 0 {
        let reg = builder.register(remains_len).unwrap();
        let mut val =1;
        for _ in 0..remains_len-1{
            val = val<<1+1;
        }
        let init = reg.handle().make_init_from_index(val).unwrap();
        chunks.push(reg);
        inits.push(init)
    }
    let reg = builder.merge(chunks).unwrap();
    (reg, inits)
}