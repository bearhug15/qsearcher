use std::collections::HashMap;
use qip::{OpBuilder, Register};
use qip::pipeline::{RegisterInitialState};
use qip::program;
use crate::searcher::utils::{LayeredRegister, zeroed_register,swap_wrapper, register_rotr, register_shiftr, register_xor, register_sum};

pub(crate) struct WordsProvider {
    //region->word_number->words
    words: HashMap<(u64, u64), Vec<Register>>
}

impl WordsProvider {
    pub fn init() -> Self {
        WordsProvider { words: HashMap::new() }
    }
    pub fn get_word(&mut self, builder: &mut OpBuilder, regs: &mut LayeredRegister, region: u64, i: u64) -> (Register, Vec<RegisterInitialState<f32>>) {
        let region_result = self.words.get_mut(&(region, i));
        match region_result {
            Some(vec) => {
                if !vec.is_empty() {
                    (vec.pop().unwrap(), Vec::<RegisterInitialState<f32>>::with_capacity(0))
                } else {
                    let (words, mut init) = parse_data_into_words(builder, regs.pop_layer());
                    insert_words_in_map(words, &mut self.words);
                    let (reg, mut init_buff) = self.get_word(builder, regs, region, i);
                    init.append(&mut init_buff);
                    (reg, init)
                }
            }
            None => {
                match i {
                    0..=15 => {
                        let (words, mut init) = parse_data_into_words(builder, regs.pop_layer());
                        insert_words_in_map(words, &mut self.words);
                        let (reg, mut init_buff) = self.get_word(builder, regs, region, i);
                        init.append(&mut init_buff);
                        (reg, init)
                    }
                    16..=63 => {
                        return match try_generate_word(builder, &mut self.words, region, i) {
                            None => {
                                let (words, mut init) = parse_data_into_words(builder, regs.pop_layer());
                                insert_words_in_map(words, &mut self.words);
                                let (reg, mut init_buff) = self.get_word(builder, regs, region, i);
                                init.append(&mut init_buff);
                                (reg, init)
                            }
                            Some(val) => {
                                val
                            }
                        };
                    }
                    _ => { panic!("Word number not in range") }
                }
            }
        }
    }
    pub fn return_word(&mut self, word: Register, region: u64, i: u64) {
        let vec = self.words.get_mut(&(region, i)).unwrap();
        vec.push(word);
    }
}

fn try_generate_word(builder: &mut OpBuilder, full_map: &mut HashMap<(u64, u64), Vec<Register>>, region: u64, i: u64) -> Option<(Register, Vec<RegisterInitialState<f32>>)> {
    if i < 16 { panic!("Words generated only for range 16..=63") }

    let word15_vec = match full_map.get_mut(&(region, i - 15)) {
        None => { return None; }
        Some(val) => { val }
    };
    let word15 = match word15_vec.pop() {
        None => { return None; }
        Some(val) => { val }
    };
    let word2_vec = match full_map.get_mut(&(region, i - 2)) {
        None => {
            full_map.get_mut(&(region, i - 15)).unwrap().push(word15);
            return None;
        }
        Some(val) => { val }
    };
    let word2 = match word2_vec.pop() {
        None => {
            full_map.get_mut(&(region, i - 15)).unwrap().push(word15);
            return None;
        }
        Some(val) => { val }
    };
    let word16_vec = match full_map.get_mut(&(region, i - 16)) {
        None => {
            full_map.get_mut(&(region, i - 15)).unwrap().push(word15);
            full_map.get_mut(&(region, i - 2)).unwrap().push(word2);
            return None;
        }
        Some(val) => { val }
    };
    let word16 = match word16_vec.pop() {
        None => {
            full_map.get_mut(&(region, i - 15)).unwrap().push(word15);
            full_map.get_mut(&(region, i - 2)).unwrap().push(word2);
            return None;
        }
        Some(val) => { val }
    };
    let word7_vec = match full_map.get_mut(&(region, i - 7)) {
        None => {
            full_map.get_mut(&(region, i - 15)).unwrap().push(word15);
            full_map.get_mut(&(region, i - 2)).unwrap().push(word2);
            full_map.get_mut(&(region, i - 16)).unwrap().push(word16);
            return None;
        }
        Some(val) => { val }
    };
    let word7 = match word7_vec.pop() {
        None => {
            full_map.get_mut(&(region, i - 15)).unwrap().push(word15);
            full_map.get_mut(&(region, i - 2)).unwrap().push(word2);
            full_map.get_mut(&(region, i - 16)).unwrap().push(word16);
            return None;
        }
        Some(val) => { val }
    };

    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let (word15, s0, mut init_buff) = get_s0(builder, word15);
    init.append(&mut init_buff);
    let (word2, s1, mut init_buff) = get_s1(builder, word2);
    init.append(&mut init_buff);
    let (word16, s0, word7, s1, wi, mut init_buff) = get_wi(builder, word16, s0, word7, s1);
    init.append(&mut init_buff);

    let word16_vec = full_map.get_mut(&(region, i - 16)).unwrap();
    word16_vec.push(word16);
    let word15_vec = full_map.get_mut(&(region, i - 15)).unwrap();
    word15_vec.push(word15);
    let word7_vec = full_map.get_mut(&(region, i - 7)).unwrap();
    word7_vec.push(word7);
    let word2_vec = full_map.get_mut(&(region, i - 2)).unwrap();
    word2_vec.push(word2);
    Some((wi, init))
}

fn insert_words_in_map(mut words: Vec<Register>, full_map: &mut HashMap<(u64, u64), Vec<Register>>) {
    let regions_amount = words.len() / 16;
    for region in 0..regions_amount {
        let region = region as u64;
        for i in 0..16 {
            let reg = words.pop().unwrap();
            let regs = full_map.get_mut(&(region, i));
            match regs {
                None => {
                    let regs = vec![reg];
                    full_map.insert((region, i), regs);
                }
                Some(regs) => {
                    regs.push(reg);
                }
            };
        }
    }
}

fn parse_data_into_words(builder: &mut OpBuilder, mut reg: Register) -> (Vec<Register>, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let words_amount = reg.n() / 32;
    let mut words = Vec::<Register>::with_capacity(words_amount as usize);
    for i in 0..words_amount {
        let (word, mut init_buff) = zeroed_register(builder, 32);
        let (reg_buff, word_buff) = program!(builder,reg,word;
        swap_wrapper reg[i*32..(i+1)*32], word;
        ).unwrap();
        init.append(&mut init_buff);
        words.push(word_buff);
        reg = reg_buff;
    }
    (words, init)
}

fn get_s0(builder: &mut OpBuilder, reg: Register) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let (reg, buff1, mut init_buff) = register_rotr(builder, reg, 7);
    init.append(&mut init_buff);
    let (reg, buff2, mut init_buff) = register_rotr(builder, reg, 18);
    init.append(&mut init_buff);
    let (reg, buff3, mut init_buff) = register_shiftr(builder, reg, 3);
    init.append(&mut init_buff);
    let (buff1, buff2, xored1, mut init_buff) = register_xor(builder, buff1, buff2);
    init.append(&mut init_buff);
    let (xored1, buff3, xored2, mut init_buff) = register_xor(builder, xored1, buff3);
    init.append(&mut init_buff);
    (reg, xored2, init)
}

fn get_s1(builder: &mut OpBuilder, reg: Register) -> (Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let (reg, buff1, mut init_buff) = register_rotr(builder, reg, 17);
    init.append(&mut init_buff);
    let (reg, buff2, mut init_buff) = register_rotr(builder, reg, 19);
    init.append(&mut init_buff);
    let (reg, buff3, mut init_buff) = register_shiftr(builder, reg, 10);
    init.append(&mut init_buff);
    let (buff1, buff2, xored1, mut init_buff) = register_xor(builder, buff1, buff2);
    init.append(&mut init_buff);
    let (xored1, buff3, xored2, mut init_buff) = register_xor(builder, xored1, buff3);
    init.append(&mut init_buff);
    (reg, xored2, init)
}

fn get_wi(builder: &mut OpBuilder, w16: Register, s0: Register, w7: Register, s1: Register) -> (Register, Register, Register, Register, Register, Vec<RegisterInitialState<f32>>) {
    let mut init = Vec::<RegisterInitialState<f32>>::new();
    let (w16, s0, buff1, mut init_buff) = register_sum(builder, w16, s0);
    init.append(&mut init_buff);
    let (buff1, w7, buff2, mut init_buff) = register_sum(builder, buff1, w7);
    init.append(&mut init_buff);
    let (buff2, s1, buff3, mut init_buff) = register_sum(builder, buff2, s1);
    init.append(&mut init_buff);
    (w16, s0, w7, s1, buff3, init)
}
