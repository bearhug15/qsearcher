/*
struct st{}
impl st{
    fn call1(&self, message: &str){
        println!("call1 {}",message);
    }
    fn call2(&self, message: &str){
        println!("call2 {}",message);
    }
}
struct bb{}
impl bb{
    fn apply(&self, mut f: Box<FnMut(& st, &str)>, st_use: &st, mes: &str){
        f(st_use,mes);
    }
}*/
use std::mem::{size_of, transmute};
use qip::{OpBuilder, UnitaryBuilder, Register, CircuitError, run_local_with_init, Complex};
use qip::program;
//use crate::searcher::utils::blank;
use std::convert::TryInto;

fn main() {

    let n = 3;
    let mut bb = OpBuilder::new();
    let mut b :&mut dyn UnitaryBuilder =&mut bb;
    let mut b  = Box::from(b);
    //let mut b:&mut [u8] = unsafe{ transmute(b)};
    let mut b: Box<&mut OpBuilder> = unsafe{transmute(b)};
    let mut b =*b;
    let ra = b.register(n).unwrap();
    let rb = b.register(n).unwrap();

    /*fn gamma(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
        let rb = rs.pop().unwrap();
        let ra = rs.pop().unwrap();
        let q = b.register(2).unwrap();
        let q = b.hadamard(q);
        let (q, ra, rb) = b.cswap(q, ra, rb)?;
        //let q = b.hadamard(q);
        Ok(vec![ra, rb])
    }
    fn ccnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
        let r3 = rs.pop().unwrap();
        let r2 = rs.pop().unwrap();
        let r1 = rs.pop().unwrap();
        let(r1,r2,r3) = b.ccnot(r1,r2,r3);
        Ok(vec![r1,r2,r3])
    }

    let mut reg1 = b.register(n).unwrap();
    let mut reg2 = b.register(n).unwrap();
    let mut res = b.register(n).unwrap();

    let (reg1,reg2) = program!(b,reg1,reg2;
    swap_wrapper reg1[0], reg2[2];
    swap_wrapper reg1[1..=2], reg2[0..=1];
    ).unwrap();
    let init = vec![reg1.handle().make_init_from_index(0b101).unwrap(),
                    reg2.handle().make_init_from_index(0b000).unwrap(),
                    res.handle().make_init_from_index(0).unwrap()];
    let (reg2,m_handle) = b.measure(reg2);
    let (_,measured) = run_local_with_init::<f32>(&reg2,&init).unwrap();
    let (result,_) = measured.get_measurement(&m_handle).unwrap();
    println!("{}",result);*/
    /*fn split(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
        let r1 = rs.pop().unwrap();
        let r2 = rs.pop().unwrap();
        Ok(vec![r1,r2])
    }

    let mut reg = b.register(6).unwrap();
    let mut reg2 = b.register(2).unwrap();
    let mut reg3 = b.register(4).unwrap();
    let init = vec![reg.handle().make_init_from_index(0b000110).unwrap()];
    let mut res = Vec::<Register>::with_capacity(3);
    let mut n = reg.n();
    for i in 0..2{
        let (buff1,buff2 )= program!(b,reg2,reg;
        split reg[i*2..(i+1)*2],reg[(i+1)*2..n];
        ).unwrap();
        n=n-2;
        println!("buff1 {}",buff1.n());
        println!("buff2 {}",buff2.n());
        reg = buff2;
        res.push(buff1);
        reg2 = b.register(2).unwrap();
        reg3 = b.register(2).unwrap();
    }
    res.push(reg);
    //let res:[Register;3] = res.try_into().unwrap();
    let res1 = res.pop().unwrap();
    let res2 = res.pop().unwrap();
    let res3 = res.pop().unwrap();
    let (res1,res1_handle) = b.measure(res1);
    let (res2,res2_handle) = b.measure(res2);
    let (res3,res3_handle) = b.measure(res3);
    let (_,measured1) = run_local_with_init::<f32>(&res1,&init).unwrap();
    println!("{:?}",measured1);
    let (_,measured2) = run_local_with_init::<f32>(&res2,&init).unwrap();
    println!("{:?}",measured2);
    let (_,measured3) = run_local_with_init::<f32>(&res3,&init).unwrap();
    println!("{:?}",measured3);
    //let r2_handle = r2.handle();
    */
    //println!("Measured: {:?} (with chance {:?})", result2, p2);
}
fn cnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
    let r1 = rs.pop().unwrap();
    //println!("{}",r1.n());
    let r2 = rs.pop().unwrap();
    //println!("{}",r2.n());
    let (r1,r2) = b.cnot(r1,r2);
    Ok(vec![b.merge(vec![r1, r2]).unwrap()])
}


fn func(mut f: Box<dyn FnMut(&mut dyn UnitaryBuilder, Vec<Register>) -> Result<Vec<Register>, CircuitError>>, builder:&mut OpBuilder,range:(usize,usize)){
    let n = 3;
    let mut b = OpBuilder::new();
    let ra = b.register(n).unwrap();
    let rb = b.register(n).unwrap();

    let a_handle = ra.handle();
    let b_handle = rb.handle();
    let initial_state = [a_handle.make_init_from_index(0b100).unwrap(),
        b_handle.make_init_from_index(0b010).unwrap()];
    let (ra, rb) = program!(&mut b, ra, rb;
    // Applies gamma to |ra[0] ra[1]>|ra[2]>
    f ra[range.0..range.1], rb[range.0..range.1];
).unwrap();
    let (ra, ram_handle) = b.measure(ra);
    let (rb, rbm_handle) = b.measure(rb);
    let r = b.merge(vec![ra, rb]).unwrap();
    let (_, measured) = run_local_with_init::<f64>(&r, &initial_state).unwrap();
    let (result1, p1) = measured.get_measurement(&ram_handle).unwrap();
    let (result2, p2) = measured.get_measurement(&rbm_handle).unwrap();
    //println!("Measured: {:?} (with chance {:?})", result1, p1);
    //println!("Measured: {:?} (with chance {:?})", result2, p2);
}
fn swap_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
    let r2 = rs.pop().unwrap();
    let r1 = rs.pop().unwrap();
    let (r1, r2) = b.swap(r1, r2).unwrap();
    Ok(vec![r1, r2])
}