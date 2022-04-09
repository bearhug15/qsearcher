#![allow(non_snake_case)]

use std::collections::HashMap;
use std::convert::TryInto;

use qip::{OpBuilder, Register, UnitaryBuilder};
use qip::pipeline::RegisterInitialState;

use crate::searcher::oracle::Oracle;
use crate::searcher::searcher::func_hadamard;
use crate::searcher::utils::{LayeredRegister, register_and, register_entangled_copy, register_eq, register_not, register_rotl, register_sum, register_xor};
use crate::sha256::words_provider::WordsProvider;

pub struct SHA256Oracle {
    //builder: OpBuilder,
    service_data: Vec<u8>,
    limit: [u8; 32],
}

impl Oracle for SHA256Oracle {
    fn set_service_data(&mut self, service_data: Option<Vec<u8>>) {
        match service_data {
            None => {
                let vars: [u32; 8] = [
                    0x6A09E667,
                    0xBB67AE85,
                    0x3C6EF372,
                    0xA54FF53A,
                    0x510E527F,
                    0x9B05688C,
                    0x1F83D9AB,
                    0x5BE0CD19
                ];
                //let u8_vars = <&[u8;32]>::try_from(&vars).unwrap();

                let vars = vars.iter().map(|val| {
                    val.to_be_bytes()
                }).fold(Vec::<u8>::with_capacity(32), |mut vec, val| {
                    vec.append(&mut val.to_vec());
                    vec
                });
                self.service_data = vars;
            }
            Some(data) => { self.service_data = data; }
        }
    }

    fn get_main_qubit_usage(&self) -> Option<usize> {
        None
    }

    fn make_prediction(&mut self, builder: &mut OpBuilder, mut main_data: LayeredRegister) -> (LayeredRegister, Register, Vec<RegisterInitialState<f32>>) {
        if main_data.width() % 512 != 0 { panic!("Length should be a multiple of 512") }
        let mut init = Vec::<RegisterInitialState<f32>>::new();
        let (mut vars, mut init1) = self.init_service(builder);
        let (mut consts, mut init2) = init_consts(builder);
        init.append(&mut init1);
        init.append(&mut init2);
        let steps = main_data.width() / 512;
        let mut words_provider = WordsProvider::init();
        for step in 0..steps {
            let (vars_buff, mut vars_copy, mut init_buff) = get_vars_copy(builder, vars);
            vars = vars_buff;
            init.append(&mut init_buff);
            for i in 0..63 {
                let mut vars_copyv = Vec::<Register>::from(vars_copy);
                let a: Register = vars_copyv.pop().unwrap();
                let b: Register = vars_copyv.pop().unwrap();
                let c: Register = vars_copyv.pop().unwrap();
                let mut d: Register = vars_copyv.pop().unwrap();
                let e: Register = vars_copyv.pop().unwrap();
                let f: Register = vars_copyv.pop().unwrap();
                let g: Register = vars_copyv.pop().unwrap();
                let mut h: Register = vars_copyv.pop().unwrap();


                let (a, Sum0, mut init_buff) = get_Sum0(builder, a);
                init.append(&mut init_buff);
                let (mut a, mut b, mut c, Ma, mut init_buff) = get_Ma(builder, a, b, c);
                init.append(&mut init_buff);
                let (t2, _, _, mut init_buff) = get_t2(builder, Sum0, Ma);
                init.append(&mut init_buff);
                let (e, Sum1, mut init_buff) = get_Sum1(builder, e);
                init.append(&mut init_buff);
                let (mut e, mut f, mut g, Ch, mut init_buff) = get_Ch(builder, e, f, g);
                init.append(&mut init_buff);
                let (wi, mut init_buff) = words_provider.get_word(builder, &mut main_data, step, i);
                init.append(&mut init_buff);
                let const_buff = consts.remove(&(i as usize)).unwrap();
                let (_, _, _, const_buff, wi, t1, mut init_buff) = get_t1(builder, h, Sum1, Ch, const_buff, wi);
                consts.insert(i as usize, const_buff);
                init.append(&mut init_buff);
                words_provider.return_word(wi, step, i);

                h = g;
                g = f;
                f = e;
                let (_, t1, res, mut init_buff) = register_sum(builder, d, t1);
                init.append(&mut init_buff);
                e = res;
                d = c;
                c = b;
                b = a;
                let (_, _, res, mut init_buff) = register_sum(builder, t1, t2);
                init.append(&mut init_buff);
                a = res;

                vars_copyv.push(a);
                vars_copyv.push(b);
                vars_copyv.push(c);
                vars_copyv.push(d);
                vars_copyv.push(e);
                vars_copyv.push(f);
                vars_copyv.push(g);
                vars_copyv.push(h);
                vars_copy = vars_copyv.try_into().unwrap();
            }
            let (vars_buff, mut init_buff) = get_next_vars(builder, vars, vars_copy);
            vars = vars_buff;
            init.append(&mut init_buff);
        }
        let res = builder.merge(Vec::from(vars)).unwrap();
        let (limit, init_buff) = self.init_limit(builder);
        init.append(&mut init_buff.clone());

        let (_, _, sign, mut init_buff2) = register_eq(builder, res, limit);
        init.append(&mut init_buff2);
        (main_data, sign, init)
    }


    fn init_main_data(&mut self, builder: &mut OpBuilder, old_data: &Vec<u8>, nonce_size: u64, main_qubit_usage: usize) -> (LayeredRegister, Vec<RegisterInitialState<f32>>) {
        let main_size_bits = old_data.len() as u64 * 8 + nonce_size * 8;
        let full_size_bits = ((main_size_bits + 8 + 64) % 512 + main_size_bits + 8 + 64) as u64;
        //LayeredRegister::new(builder,full_size_bits,main_qubit_usage);
        let mut data = old_data.clone();
        for _ in 0..nonce_size {
            data.push(0)
        }
        data.push(0b10000000);
        let additional_zeros = (main_size_bits + 8 + 64) % 512;
        for _ in 0..additional_zeros {
            data.push(0)
        }
        let mut len_data = Vec::from(main_size_bits.to_be_bytes());
        data.append(&mut len_data);
        let (mut qubits, init) = LayeredRegister::new_initialized(builder, full_size_bits, main_qubit_usage, &data);
        let h_ptr = Box::new(func_hadamard);
        qubits = qubits.apply_to_layers1_in_range(h_ptr, builder, main_size_bits, nonce_size * 8);
        qubits.set_dif_range((old_data.len() * 8) as u64, main_size_bits);
        (qubits, init)
    }
}

impl SHA256Oracle {
    fn init(limit: [u8; 32]) -> Self {
        SHA256Oracle { service_data: vec![], limit }
    }
    fn init_service(&self, builder: &mut OpBuilder) -> ([Register; 8], Vec<RegisterInitialState<f32>>) {
        if self.service_data.len() != 8 {
            panic!("Service data should have 8 starting variables")
        }
        let mut regs = Vec::<Register>::with_capacity(8);
        let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(8);
        self.service_data.iter().for_each(|val| {
            let reg = builder.register(8).unwrap();
            let initialized: RegisterInitialState<f32> = reg.handle().make_init_from_index(u64::from(val.clone())).unwrap();
            init.push(initialized);
            regs.push(reg)
        });
        let regs: [Register; 8] = regs.try_into().unwrap();
        (regs, init)
    }
    fn init_limit(&self, builder: &mut OpBuilder) -> (Register, Vec<RegisterInitialState<f32>>) {
        let mut regs = Vec::<Register>::with_capacity(32);
        let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(32);
        self.limit.iter().for_each(|val| {
            let reg = builder.register(8).unwrap();
            let initialized: RegisterInitialState<f32> = reg.handle().make_init_from_index(u64::from(val.clone())).unwrap();
            init.push(initialized);
            regs.push(reg)
        });
        let reg = builder.merge(regs).unwrap();
        (reg, init)
    }
}

fn init_consts(builder: &mut OpBuilder) -> (HashMap<usize, Register>, Vec<RegisterInitialState<f32>>) {
    let consts: [u32; 64] = [
        0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
        0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
        0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
        0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
        0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
        0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
        0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
        0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2
    ];
    let mut regs = Vec::<Register>::with_capacity(64);
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(64);
    consts.iter().for_each(|val| {
        let reg = builder.register(32).unwrap();
        let initialized: RegisterInitialState<f32> = reg.handle().make_init_from_index(u64::from(val.clone())).unwrap();
        init.push(initialized);
        regs.push(reg)
    });
    let map = HashMap::<usize, Register>::with_capacity(64);

    regs.into_iter().enumerate().fold(HashMap::<usize, Register>::with_capacity(64), |mut map, (i, val)| {
        map.insert(i, val);
        map
    });
    (map, init)
}

fn get_vars_copy(builder: &mut OpBuilder, mut init_vars: [Register; 8]) -> ([Register; 8], [Register; 8], Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(8);
    let mut vars = Vec::<Register>::with_capacity(8);
    let mut vars_copy = Vec::<Register>::with_capacity(8);
    for val in IntoIterator::into_iter(init_vars) {
        let (val1, val2, mut init_buff) = register_entangled_copy(builder, val);
        init.append(&mut init_buff);
        vars.push(val1);
        vars_copy.push(val2);
    };
    /*init_vars.into_iter().for_each(
        |val:Register|{
            let (val1, val2, mut init_buff) = register_entangled_copy(builder, val);
            init.append(&mut init_buff);
            vars.push(val1);
            vars_copy.push(val2);
        });*/
    let vars: [Register; 8] = vars.try_into().unwrap();
    let vars_copy: [Register; 8] = vars_copy.try_into().unwrap();
    (vars, vars_copy, init)
}

fn get_Sum0(builder: &mut OpBuilder, a: Register) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(5);
    let (a, buff1, mut init_buff) = register_rotl(builder, a, 2);
    init.append(&mut init_buff);
    let (a, buff2, mut init_buff) = register_rotl(builder, a, 13);
    init.append(&mut init_buff);
    let (a, buff3, mut init_buff) = register_rotl(builder, a, 22);
    init.append(&mut init_buff);
    let (_, _, xored1, mut init_buff) = register_xor(builder, buff1, buff2);
    init.append(&mut init_buff);
    let (_, _, xored2, mut init_buff) = register_xor(builder, xored1, buff3);
    init.append(&mut init_buff);
    (a, xored2, init)
}

fn get_Ma(builder: &mut OpBuilder, a: Register, b: Register, c: Register) -> (Register, Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(5);
    let (mut a, mut b, mut ab, mut init_buff) = register_and(builder, a, b);
    init.append(&mut init_buff);
    let (mut a, mut c, mut ac, mut init_buff) = register_and(builder, a, c);
    init.append(&mut init_buff);
    let (mut b, mut c, mut bc, mut init_buff) = register_and(builder, b, c);
    init.append(&mut init_buff);
    let (_, _, xored1, mut init_buff) = register_xor(builder, ab, ac);
    init.append(&mut init_buff);
    let (_, _, xored2, mut init_buff) = register_xor(builder, xored1, bc);
    init.append(&mut init_buff);
    (a, b, c, xored2, init)
}

fn get_t2(builder: &mut OpBuilder, Sum0: Register, Ma: Register) -> (Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    register_sum(builder, Sum0, Ma)
}

fn get_Sum1(builder: &mut OpBuilder, e: Register) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(5);
    let (e, buff1, mut init_buff) = register_rotl(builder, e, 6);
    init.append(&mut init_buff);
    let (e, buff2, mut init_buff) = register_rotl(builder, e, 11);
    init.append(&mut init_buff);
    let (e, buff3, mut init_buff) = register_rotl(builder, e, 25);
    init.append(&mut init_buff);
    let (_, _, xored1, mut init_buff) = register_xor(builder, buff1, buff2);
    init.append(&mut init_buff);
    let (_, _, xored2, mut init_buff) = register_xor(builder, xored1, buff3);
    init.append(&mut init_buff);
    (e, xored2, init)
}

fn get_Ch(builder: &mut OpBuilder, e: Register, f: Register, g: Register) -> (Register, Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(4);
    let (e, f, ef, mut init_buff) = register_and(builder, e, f);
    init.append(&mut init_buff);
    let (e, not, mut init_buff) = register_not(builder, e);
    init.append(&mut init_buff);
    let (_, g, notg, mut init_buff) = register_and(builder, not, g);
    init.append(&mut init_buff);
    let (_, _, xored, mut init_buff) = register_xor(builder, ef, notg);
    init.append(&mut init_buff);
    (e, f, g, xored, init)
}

fn get_t1(builder: &mut OpBuilder, h: Register, Sum1: Register, Ch: Register, k: Register, w: Register) -> (Register, Register, Register, Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(4);
    let (h, Sum1, s1, mut init_buff) = register_sum(builder, h, Sum1);
    init.append(&mut init_buff);
    let (_, Ch, s2, mut init_buff) = register_sum(builder, s1, Ch);
    init.append(&mut init_buff);
    let (_, k, s3, mut init_buff) = register_sum(builder, s2, k);
    init.append(&mut init_buff);
    let (_, w, s4, mut init_buff) = register_sum(builder, s3, w);
    init.append(&mut init_buff);
    (h, Sum1, Ch, k, w, s4, init)
}

fn get_next_vars(builder: &mut OpBuilder, mut vars: [Register; 8], mut increment: [Register; 8]) -> ([Register; 8], Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let mut result = Vec::<Register>::with_capacity(8);
    for (var, inc) in IntoIterator::into_iter(vars).zip(IntoIterator::into_iter(increment)) {
        let (_, _, res, mut init_buff) = register_sum(builder, var, inc);
        init.append(&mut init_buff);
        result.push(res);
    };

    let result: [Register; 8] = result.try_into().unwrap();

    (result, init)
}

