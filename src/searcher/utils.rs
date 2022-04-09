use qip::{CircuitError, OpBuilder, Register, UnitaryBuilder};
use qip::pipeline::RegisterInitialState;
use qip::program;

pub struct LayeredRegister {
    layers: Vec<Register>,
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
        let func = move |builder: &mut dyn UnitaryBuilder, mut regs: Vec<Register>| -> Result<Vec<Register>, CircuitError>{
            let reg = regs.pop().unwrap();
            Ok(vec![f(builder, reg)])
        };
        let layers: Vec<Register> = self.layers.into_iter().map(|reg| {
            program!(builder,reg;
            func reg[start..end];
            ).unwrap()
        }).collect();
        LayeredRegister { layers, current_layer_index, start_depth, width, dif_range }
    }

    pub(crate) fn mirror_in_range(mut self, builder: &mut OpBuilder, sign: Register, start: u64, end: u64) -> (Self, Register) {
        let current_layer_index = self.current_layer_index;
        let start_depth = self.start_depth;
        let width = self.width;
        let dif_range = self.dif_range;
        let len = self.layers.len();
        let (sign, layers) = self.layers.into_iter().fold((sign, Vec::<Register>::with_capacity(len)), |(sign, mut layers), reg| {
            let (sign_buff, reg_buff) = program!(builder,sign,reg;
            cnot_wrapper sign, reg[start..end];
            ).unwrap();
            layers.push(reg_buff);
            (sign_buff, layers)
        });
        (LayeredRegister { layers, current_layer_index, start_depth, width, dif_range }, sign)
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

pub fn register_rotr(builder: &mut OpBuilder, mut reg: Register, mut step: u64) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg.n();
    step = step % n;
    let (res, init) = zeroed_register(builder, n);
    let (reg, res) = program!(builder,reg,res;
    cnot_wrapper reg[0..n-step], res[step..n];
    cnot_wrapper reg[n-step..n], res[0..step];
    ).unwrap();
    (reg, res, init)
}

pub fn register_rotr_inplace(builder: &mut OpBuilder, reg: Register, mut step: u64) -> Register {
    let n = reg.n();
    step = step % n;
    let (res, _) = zeroed_register(builder, n);
    let (_, res) = program!(builder,reg,res;
    swap_wrapper reg[0..n-step], res[step..n];
    swap_wrapper reg[n-step..n], res[0..step];
    ).unwrap();
    res
}

pub fn register_rotl(builder: &mut OpBuilder, reg: Register, mut step: u64) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg.n();
    step = step % n;
    let (res, init) = zeroed_register(builder, n);
    let (reg, res) = program!(builder,reg,res;
    cnot_wrapper reg[step..n], res[0..n-step];
    cnot_wrapper reg[0..step], res[n-step..n];
    ).unwrap();
    (reg, res, init)
}

pub fn register_rotl_inplace(builder: &mut OpBuilder, reg: Register, mut step: u64) -> Register {
    let n = reg.n();
    step = step % n;
    let (res, _) = zeroed_register(builder, n);
    let (_, res) = program!(builder,reg,res;
    swap_wrapper reg[step..n], res[0..n-step];
    swap_wrapper reg[0..step], res[n-step..n];
    ).unwrap();
    res
}

pub fn register_and(builder: &mut OpBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
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
    (reg1, reg2, res, init)
}

pub fn register_or(builder: &mut OpBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
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
    (reg1, reg2, res, init)
}

pub fn register_xor(builder: &mut OpBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg1.n();
    reg1 = builder.not(reg1);
    let (mut res, mut init) = zeroed_register(builder, n);
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
    for i in 0..n {
        let (buff1, buff2, res_buff) = program!(builder, reg1, reg2,res;
            ccnot_wrapper reg1[i],reg2[i],res[i];
        ).unwrap();
        reg1 = buff1;
        reg2 = buff2;
        res = res_buff;
    }
    reg2 = builder.not(reg2);
    (reg1, reg2, res, init)
}

pub fn register_shiftr(builder: &mut OpBuilder, mut reg: Register, mut step: u64) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg.n();
    let (mut res, init) = zeroed_register(builder, n);
    for i in 0..n - step {
        let (buff1, res_buff) = program!(builder, reg,res;
            cnot_wrapper reg[i], res[i+step];
        ).unwrap();
        reg = buff1;
        res = res_buff;
    }
    /*let (reg, res) = program!(builder,reg,res;
    cnot reg[0..n-step], res[step..n];
    ).unwrap();*/
    (reg, res, init)
}

pub fn register_shiftl(builder: &mut OpBuilder, mut reg: Register, mut step: u64) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg.n();
    let (mut res, init) = zeroed_register(builder, n);
    for i in step..n {
        let (buff1, res_buff) = program!(builder, reg,res;
            cnot_wrapper reg[i], res[i-step];
        ).unwrap();
        reg = buff1;
        res = res_buff;
    }
    /*let (reg, res) = program!(builder,reg,res;
    cnot reg[step..n], res[0..n-step];
    ).unwrap();*/
    (reg, res, init)
}

pub fn register_not(builder: &mut OpBuilder, mut reg: Register) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg.n();
    let (mut res, init) = oneed_register(builder, n);
    for i in 0..n {
        let (buff, res_buff) = program!(builder, reg,res;
            cnot_wrapper reg[i],res[i];
        ).unwrap();
        reg = buff;
        res = res_buff;
    }
    (reg, res, init)
}

pub fn register_entangled_copy(b: &mut dyn UnitaryBuilder, mut reg: Register) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg.n();
    let (mut res, init) = zeroed_register(b, n);
    for i in 0..n {
        let (buff, res_buff) = program!(b, reg,res;
            cnot_wrapper reg[i],res[i];
        ).unwrap();
        reg = buff;
        res = res_buff;
    }
    (reg, res, init)
}

pub fn ccnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let r3 = rs.pop().unwrap();
    let r2 = rs.pop().unwrap();
    let r1 = rs.pop().unwrap();
    let (r1, r2, r3) = b.ccnot(r1, r2, r3);
    Ok(vec![r1, r2, r3])
}

pub fn cnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let r2 = rs.pop().unwrap();
    let r1 = rs.pop().unwrap();
    let (r1, r2) = b.cnot(r1, r2);
    Ok(vec![r1, r2])
}

pub fn swap_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let r2 = rs.pop().unwrap();
    let r1 = rs.pop().unwrap();
    let (r1, r2) = b.swap(r1, r2).unwrap();
    Ok(vec![r1, r2])
}

pub fn zeroed_register(b: &mut dyn UnitaryBuilder, len: u64) -> (Register, Vec<RegisterInitialState<f32>>) {
    let chunks_amount = len / 8;
    let remains_len = len % 8;
    let buff = chunks_amount + if remains_len != 0 { 1 } else { 0 };
    let mut chunks = Vec::<Register>::with_capacity(buff as usize);
    let mut inits = Vec::<RegisterInitialState<f32>>::with_capacity(buff as usize);
    for _ in 0..chunks_amount {
        let reg = b.register(8).unwrap();
        let init = reg.handle().make_init_from_index(0).unwrap();
        chunks.push(reg);
        inits.push(init)
    }
    if remains_len != 0 {
        let reg = b.register(remains_len).unwrap();
        let init = reg.handle().make_init_from_index(0).unwrap();
        chunks.push(reg);
        inits.push(init)
    }
    let reg = b.merge(chunks).unwrap();
    (reg, inits)
}

pub fn oneed_register(b: &mut dyn UnitaryBuilder, len: u64) -> (Register, Vec<RegisterInitialState<f32>>) {
    let (mut reg, init) = zeroed_register(b, len);
    reg = b.not(reg);
    (reg, init)
}

pub fn register_sum(builder: &mut OpBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg1.n();
    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let (mut res, mut init1) = zeroed_register(builder, n);
    init.append(&mut init1);
    let (mut p, mut init1) = zeroed_register(builder, 1);
    init.append(&mut init1);
    let (mut c1, mut init1) = zeroed_register(builder, 1);
    init.append(&mut init1);
    let (mut c2, mut init1) = zeroed_register(builder, 1);
    init.append(&mut init1);
    let (mut c3, mut init1) = zeroed_register(builder, 1);
    init.append(&mut init1);
    for i in 0..n {
        let (reg1_buff, reg2_buff, res_buff, p_buff, c1_buff, c2_buff, c3_buff)
            = program!(builder,reg1,reg2,res,p,c1,c2,c3;
        sum_wrapper reg1[i],reg2[i],res[i],p,c1,c2,c3;
        ).unwrap();
        reg1 = reg1_buff;
        reg2 = reg2_buff;
        res = res_buff;
        p = p_buff;
        let (mut c1_buff, mut init1) = zeroed_register(builder, 1);
        c1 = c1_buff;
        init.append(&mut init1);
        let (mut c2_buff, mut init1) = zeroed_register(builder, 1);
        c2 = c2_buff;
        init.append(&mut init1);
        let (mut c3_buff, mut init1) = zeroed_register(builder, 1);
        c3 = c3_buff;
        init.append(&mut init1);
    }
    (reg1, reg2, res, init)
}

pub fn sum_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let mut c1 = rs.pop().unwrap();
    let mut c2 = rs.pop().unwrap();
    let mut c3 = rs.pop().unwrap();
    let mut p = rs.pop().unwrap();
    let mut res = rs.pop().unwrap();
    let mut reg2 = rs.pop().unwrap();
    let mut reg1 = rs.pop().unwrap();
    let (reg1, reg2, c1) = qubit_xor(b, reg1, reg2, c1);
    let (c1, p, res) = qubit_xor(b, c1, p, res);
    let (c1, p, c2) = qubit_and(b, c1, p, c2);
    let (reg1, reg2, c3) = qubit_and(b, reg1, reg2, c3);
    let (c2, c3, p) = qubit_or(b, c2, c3, p);
    Ok(vec![reg1, reg2, res, p, c1, c2, c3])
}

pub fn qubit_xor(b: &mut dyn UnitaryBuilder, mut reg1: Register, mut reg2: Register, mut zeroed_res: Register) -> (Register, Register, Register) {
    reg1 = b.not(reg1);
    let (mut reg1, mut reg2, mut zeroed_res) = b.ccnot(reg1, reg2, zeroed_res);
    reg1 = b.not(reg1);
    reg2 = b.not(reg2);
    let (mut reg1, mut reg2, mut zeroed_res) = b.ccnot(reg1, reg2, zeroed_res);
    reg2 = b.not(reg2);
    (reg1, reg2, zeroed_res)
}

pub fn qubit_and(b: &mut dyn UnitaryBuilder, mut reg1: Register, mut reg2: Register, mut zeroed_res: Register) -> (Register, Register, Register) {
    b.ccnot(reg1, reg2, zeroed_res)
}

pub fn qubit_or(b: &mut dyn UnitaryBuilder, mut reg1: Register, mut reg2: Register, mut zeroed_res: Register) -> (Register, Register, Register) {
    reg1 = b.not(reg1);
    reg2 = b.not(reg2);
    let (mut reg1, mut reg2, zeroed_res) = b.ccnot(reg1, reg2, zeroed_res);
    reg1 = b.not(reg1);
    reg2 = b.not(reg2);
    (reg1, reg2, zeroed_res)
}

pub fn register_eq(b: &mut dyn UnitaryBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let n = reg1.n();
    let (mut res, mut init) = zeroed_register(b, n);
    reg1 = b.not(reg1);
    for i in 0..n {
        let (buff1, buff2, res_buff) = program!(b, reg1, reg2,res;
            ccnot_wrapper reg1[i],reg2[i],res[i];
        ).unwrap();
        reg1 = buff1;
        reg2 = buff2;
        res = res_buff;
    }
    reg1 = b.not(reg1);
    reg2 = b.not(reg2);
    for i in 0..n {
        let (buff1, buff2, res_buff) = program!(b, reg1, reg2,res;
            ccnot_wrapper reg1[i],reg2[i],res[i];
        ).unwrap();
        reg1 = buff1;
        reg2 = buff2;
        res = res_buff;
    }
    reg2 = b.not(reg2);
    res = b.not(res);
    let (mut fres, mut init1) = zeroed_register(b, 1);
    init.append(&mut init1);
    let (res, fres) = b.cnot(res, fres);
    (reg1, reg2, fres, init)
}

pub fn register_more_eq(builder: &mut dyn UnitaryBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
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
    reg1 = builder.not(reg1);
    for i in 0..n {
        let (buff1, buff2, res_buff) = program!(builder, reg1, reg2,res;
            ccnot_wrapper reg1[i],reg2[i],res[i];
        ).unwrap();
        reg1 = buff1;
        reg2 = buff2;
        res = res_buff;
    }
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
    let (mut fres, mut init1) = zeroed_register(builder, 1);
    init.append(&mut init1);
    let (res, fres) = builder.cnot(res, fres);
    (reg1, reg2, fres, init)
}

pub fn register_more(b: &mut dyn UnitaryBuilder, mut reg1: Register, mut reg2: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let (reg1, reg2, res1, mut init) = register_more_eq(b, reg1, reg2);
    let (reg1, reg2, res2, mut init_buff) = register_eq(b, reg1, reg2);
    init.append(&mut init_buff);
    let res2 = b.not(res2);
    let (mut res, mut init_buff) = zeroed_register(b, 1);
    init.append(&mut init_buff);
    let (res1, res2, res) = b.ccnot(res1, res2, res);
    (reg1, reg2, res, init)
}
