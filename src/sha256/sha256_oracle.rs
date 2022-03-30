use crate::searcher::oracle::Oracle;
use qip::{OpBuilder, Register, Complex, UnitaryBuilder, CircuitError};
use crate::searcher::utils::LayeredRegister;
use qip::pipeline::RegisterInitialState;
use std::convert::{TryFrom, TryInto};
use crate::searcher::searcher::func_hadamard;


pub struct SHA256Oracle{
    //builder: OpBuilder,
    service_data:Vec<u8>
}

impl Oracle for SHA256Oracle{
    fn set_service_data(&mut self, service_data: Option<Vec<u8>>) {
        match service_data{
            None => {
                let vars: [u32;8] = [
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

                let vars = vars.into_iter().map(|val| {
                    val.to_be_bytes()
                }).fold(Vec::<u8>::with_capacity(32),|mut vec,val|{
                    vec.append(&mut val.to_vec());
                    vec
                });
                self.service_data=vars;

            }
            Some(data) => {self.service_data=data;}
        }
    }

    fn get_main_qubit_usage(&self) -> Option<usize> {
        None
    }

    fn make_prediction(&self,builder: &mut OpBuilder, mut main_data: LayeredRegister) -> (LayeredRegister, Register, Vec<RegisterInitialState<f32>>) {
        if main_data.width() %512 !=0 {panic!("Length should be a multiple of 512")}
        let working_reg = main_data.pop_layer();
        let (vars,init1) = init_consts(builder);
        let (consts,init2) = init_consts(builder);
        let steps = main_data.width()/512;
        for step in 0..steps{

        }
        unimplemented!()
    }


    fn init_main_data(&mut self,builder: &mut OpBuilder, old_data: &Vec<u8>, nonce_size: u64, main_qubit_usage:usize) -> (LayeredRegister,Vec<RegisterInitialState<f32>>) {
        let main_size_bits = old_data.len() as u64 *8 + nonce_size*8;
        let full_size_bits = ((main_size_bits+8+64)%512 +main_size_bits+8+64) as u64;
        //LayeredRegister::new(builder,full_size_bits,main_qubit_usage);
        let mut data = old_data.clone();
        for _ in 0..nonce_size  {
            data.push(0)
        }
        data.push(0b10000000);
        let additional_zeros = (main_size_bits+8+64)%512;
        for _ in 0..additional_zeros {
            data.push(0)
        }
        let mut len_data =Vec::from(main_size_bits.to_be_bytes());
        data.append(&mut len_data);
        let (mut qubits,init) =LayeredRegister::new_initialized(builder,full_size_bits,main_qubit_usage,&data);
        let h_ptr = Box::new(func_hadamard);
        qubits = qubits.apply_to_layers1_in_range(h_ptr,builder,main_size_bits,nonce_size*8);
        qubits.set_dif_range((old_data.len()*8) as u64,main_size_bits);
        (qubits,init)
    }
}
impl SHA256Oracle {
    fn init_service(&self,builder:&mut OpBuilder)->([Register;8],Vec<RegisterInitialState<f32>>){
        if self.service_data.len() !=8{
            panic!("Service data should have 8 starting variables")
        }
        let mut regs=Vec::<Register>::with_capacity(8);
        let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(8);
        self.service_data.iter().for_each(|val|{
            let reg =builder.register(8).unwrap();
            let initialized:RegisterInitialState<f32> = reg.handle().make_init_from_index(u64::from(val.clone())).unwrap();
            init.push(initialized);
            regs.push(reg)
        });
        let regs:[Register;8] = regs.try_into().unwrap();
        (regs,init)
    }
}
fn init_consts(builder: &mut OpBuilder)->([Register;64],Vec<RegisterInitialState<f32>>){
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
    let mut regs=Vec::<Register>::with_capacity(64);
    let mut init = Vec::<RegisterInitialState<f32>>::with_capacity(64);
    consts.iter().enumerate().for_each(|(i,val)|{
        let reg =builder.register(32).unwrap();
        let initialized:RegisterInitialState<f32> = reg.handle().make_init_from_index(u64::from(consts[i])).unwrap();
        init.push(initialized);
        regs.push(reg)
    });
    let regs:[Register;64] = regs.try_into().unwrap();
    (regs,init)
}

