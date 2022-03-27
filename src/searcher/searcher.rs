use crate::searcher::data_preparer::DataPreparer;
use crate::searcher::oracle::Oracle;
use crate::searcher::checker::Checker;
use crate::searcher::utils::LayeredRegister;
use std::f64::consts::PI;
use std::mem::transmute;
use qip::{UnitaryBuilder, Register, OpBuilder, CircuitError};
use std::str::from_boxed_utf8_unchecked;
use qip::program;

const DEFAULT_MAIN_QUBIT_USAGE: usize = 1000;

pub fn search(mut data: Vec<u8>, data_preparer: Option<Box<dyn DataPreparer>>, mut oracle: Box<dyn Oracle>, checker: Option<Box<dyn Checker>>, start_nonce_size: Option<u64>, step: Option<u64>) -> Vec<u8> {
    match data_preparer {
        Some(prep) => {
            let prepared = prep.prepare(&data);
            let service_data = prepared.service_data;
            data = prepared.data_remains;
            oracle.set_service_data(Some(service_data));
        }
        None => oracle.set_service_data(None)
    };

    let main_qubit_usage = match oracle.get_main_qubit_usage() {
        Some(val) => val,
        None => {
            //todo add a compile time warning
            DEFAULT_MAIN_QUBIT_USAGE
        }
    };
    let epoch = 0;
    let start_nonce_size = match start_nonce_size {
        None => { 1 }
        Some(val) => { val }
    };
    let step = match step {
        None => { 1 }
        Some(val) => { val }
    };
    loop {
        let mut builder = OpBuilder::new();
        let nonce_size_bits = (start_nonce_size + epoch * step) * 8 as u64;
        let main_size_bits = (data.len() * 8) as u64 + nonce_size_bits;
        let mut main_qubits = oracle.init_main_data(&mut builder, &data, nonce_size_bits, main_qubit_usage);
        let iterations_amount = (PI * 2_u32.pow((nonce_size_bits / 2 - 2) as u32) as f64).floor() as u64;
        let nonce_range = match main_qubits.get_dif_range() {
            None => { panic!("No diffusion range") }
            Some(val) => { val }
        };

        let h_ptr: Box<dyn Fn(&mut dyn UnitaryBuilder, Register) -> Register> = Box::new(func_h);
        main_qubits = main_qubits.apply_to_layers1_in_range(h_ptr, &mut builder, nonce_range.0, nonce_range.1);

        for _ in 0..iterations_amount {
            let (buff, res) = oracle.make_prediction(main_qubits);
            main_qubits = buff;
            //todo mirroring on (-1) based on res
            let dif = Box::from(diffuse);
            main_qubits = main_qubits.apply_to_layers1_in_range(dif,&mut builder, nonce_range.0, nonce_range.1);
        }
    }
    unimplemented!()
}

fn diffuse(builder: &mut dyn UnitaryBuilder, mut qubits: Register) -> Register {
    qubits = builder.hadamard(qubits);
    qubits = builder.x(qubits);
    let len = qubits.n();
    qubits = program!(builder,qubits;
    hadamard_wrapper qubits;
    x_wrapper qubits;
    hadamard_wrapper qubits[len-1];
    cnot_wrapper qubits[0], qubits[1..len];
    hadamard_wrapper qubits[len-1];
    x_wrapper qubits;
    hadamard_wrapper qubits;
    ).unwrap();
    qubits
    //handle.make_init_from_index()
    //unimplemented!()
}

fn hadamard_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let mut reg = rs.pop().unwrap();
    reg = b.hadamard(reg);
    Ok(vec![reg])
}

fn x_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let mut reg = rs.pop().unwrap();
    reg = b.x(reg);
    Ok(vec![reg])
}

fn cnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
    let r1 = rs.pop().unwrap();
    println!("{}",r1.n());
    let r2 = rs.pop().unwrap();
    println!("{}",r2.n());
    let (r1,r2) = b.cnot(r1,r2);
    Ok(vec![b.merge(vec![r1, r2]).unwrap()])
}

fn func_h(builder: &mut dyn UnitaryBuilder, reg: Register) -> Register {
    let builder :Box<&mut dyn UnitaryBuilder>= Box::from(builder);
    let mut builder: Box<&mut OpBuilder> = unsafe{transmute(builder)};
    let mut builder = *builder;
    OpBuilder::hadamard(builder, reg)
}





