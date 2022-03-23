mod sha256;
mod searcher;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        /*let v:Vec<u8> = Vec::from([0;10]);
        for i in v.chunks(4){
            println!("{}",i.len());
        }*/
        /*let mut buff:&mut [u8;4] = &mut [0;4];
        buff[0]=1;
        println!("{:?}",buff);*/
        //println!("{}",u32::MAX.overflowing_add(1).0);
        /*let a=1;
        let b= 2;
        let c = a^b;
        println!("{} {} {}",a,b,c);*/
        let a=u8::MAX;
        let b=1;
        println!("{}",a);
        println!("{}",a^b);
        assert_eq!(2 + 2, 4);
    }
}
