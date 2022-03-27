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
use qip::{OpBuilder, UnitaryBuilder, Register, CircuitError, run_local_with_init};
use qip::program;

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

    fn gamma(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError> {
        let rb = rs.pop().unwrap();
        let ra = rs.pop().unwrap();
        let q = b.register(2).unwrap();
        let q = b.hadamard(q);
        let (q, ra, rb) = b.cswap(q, ra, rb)?;
        //let q = b.hadamard(q);
        Ok(vec![ra, rb])
    }
    //func(Box::new(gamma),&mut b,(1,3));
    /*let a_handle = ra.handle();
    let b_handle = rb.handle();
    let initial_state = [a_handle.make_init_from_index(0b100).unwrap(),
        b_handle.make_init_from_index(0b010).unwrap()];
    let (ra, rb) = program!(&mut b, ra, rb;
    // Applies gamma to |ra[0] ra[1]>|ra[2]>
    gamma ra[1..3], rb[1..3];
).unwrap();
    let (ra, ram_handle) = b.measure(ra);
    let (rb, rbm_handle) = b.measure(rb);
    let r = b.merge(vec![ra, rb]).unwrap();
    let (_, measured) = run_local_with_init::<f64>(&r, &initial_state).unwrap();
    let (result1, p1) = measured.get_measurement(&ram_handle).unwrap();
    let (result2, p2) = measured.get_measurement(&rbm_handle).unwrap();
    println!("Measured: {:?} (with chance {:?})", result1, p1);
    println!("Measured: {:?} (with chance {:?})", result2, p2);*/
    /*let a = vec![1,1];
    let b = vec![1,2];
    println!("{}",a>b);*/
    /*let mut s = st{};
    let mut b = bb{};
    b.apply(Box::new(st::call1),&s,"message")*/
    let r1 = b.register(3).unwrap();
    let r2 = b.qubit();
    let r1_handle = r1.handle();

    //let r2_handle = r2.handle();

    let r1 = program!(b,r1;
    cnot_wrapper r1[0],r1[1..=2];

    ).unwrap();

    println!("{}",r1.n());
    //println!("{}",r2.n());
    //let (r1,r1m_handle) = b.measure(r1);
    //let r1 = b.merge(vec![r1,r2]).unwrap();
    let (r1,r1m_handle) = b.measure(r1);
    let initial_state = [r1_handle.make_init_from_index(0b010).unwrap()];
    //let (r2,r2m_handle) = b.measure(r2);
    //let r = b.merge(vec![r1, r2]).unwrap();
    let (_, measured) = run_local_with_init::<f64>(&r1, &initial_state).unwrap();
    let (result1, p1) = measured.get_measurement(&r1m_handle).unwrap();
    //let (result2, p2) = measured.get_measurement(&r2m_handle).unwrap();
    println!("Measured: {:?} (with chance {:?})", result1, p1);
    //println!("Measured: {:?} (with chance {:?})", result2, p2);
}
fn cnot_wrapper(b: &mut dyn UnitaryBuilder, mut rs: Vec<Register>) -> Result<Vec<Register>, CircuitError>{
    let r1 = rs.pop().unwrap();
    println!("{}",r1.n());
    let r2 = rs.pop().unwrap();
    println!("{}",r2.n());
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
    println!("Measured: {:?} (with chance {:?})", result1, p1);
    println!("Measured: {:?} (with chance {:?})", result2, p2);
}