use std::f64::consts::PI;
use std::mem::transmute;

use qip::{CircuitError, OpBuilder, Register, UnitaryBuilder};
use qip::pipeline::RegisterInitialState;
use qip::program;

use crate::searcher::checker::Checker;
use crate::searcher::data_preparer::DataPreparer;
use crate::searcher::oracle::Oracle;
use crate::searcher::utils::LayeredRegister;

const DEFAULT_MAIN_QUBIT_USAGE: usize = 10;

pub fn search(mut data: Vec<u8>,
              data_preparer: Option<Box<dyn DataPreparer>>,
              mut oracle: Box<dyn Oracle>,
              checker: Option<Box<dyn Checker>>,
              start_nonce_size: Option<u64>,
              step: Option<u64>) -> Vec<u8> {
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
            DEFAULT_MAIN_QUBIT_USAGE
        }
    };
    let mut epoch = 0;
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
        let nonce_size_bits = (start_nonce_size + epoch * step) as u64;
        let main_size_bits = (data.len() * 8) as u64 + nonce_size_bits;
        let iterations_amount = (PI * (nonce_size_bits as f64).sqrt() as f64 /4.0).floor() as u64;
        let (mut main_qubits, mut initial_state) = oracle.init_main_data(&mut builder, &data, nonce_size_bits, main_qubit_usage * iterations_amount as usize +1);

        let nonce_range = match main_qubits.get_dif_range() {
            None => { panic!("No diffusion range") }
            Some(val) => { val }
        };

        let h_ptr: Box<dyn Fn(&mut dyn UnitaryBuilder, Register) -> Register> = Box::new(func_hadamard);
        main_qubits = main_qubits.apply_to_layers1_in_range(h_ptr, &mut builder, nonce_range.0, nonce_range.1);

        for _ in 0..iterations_amount {
            let (buff, res, mut init) = oracle.make_prediction(&mut builder, main_qubits);
            initial_state.append(&mut init);
            main_qubits = buff;
            let (buff, sign) = main_qubits.mirror_in_range(&mut builder, res, nonce_range.0, nonce_range.1);
            main_qubits = buff;
            let dif = Box::from(diffuse);
            main_qubits = main_qubits.apply_to_layers1_in_range(dif, &mut builder, nonce_range.0, nonce_range.1);
        }

        let result =  get_result(&mut builder, main_qubits,initial_state);

        match checker {
            Some(ref check) => {
                let res = check.check_nonce(result.clone());
                if res {
                    return result;
                }
            }
            None => return result
        };
        epoch = epoch + 1;
    }
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

fn cnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
    let r1 = rs.pop().unwrap();
    println!("{}", r1.n());
    let r2 = rs.pop().unwrap();
    println!("{}", r2.n());
    let (r1, r2) = b.cnot(r1, r2);
    Ok(vec![b.merge(vec![r1, r2]).unwrap()])
}

pub(crate) fn func_hadamard(builder: &mut dyn UnitaryBuilder, reg: Register) -> Register {
    let builder: Box<&mut dyn UnitaryBuilder> = Box::from(builder);
    let mut builder: Box<&mut OpBuilder> = unsafe { transmute(builder) };
    let mut builder = *builder;
    OpBuilder::hadamard(builder, reg)
}

#[allow(unused_doc_comments)]
fn get_result(builder:&mut OpBuilder,main_qubits: LayeredRegister, init: Vec<RegisterInitialState<f32>>)->Vec<u8>{
    ///This is plug for future implementation of translator to real quantum machine code
    unimplemented!()
}






