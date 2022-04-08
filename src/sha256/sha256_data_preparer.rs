use std::convert::TryFrom;
use std::mem;

use crate::searcher::data_preparer::{DataPreparer, DataPrepareResult};

pub struct SHA256DataPreparer {}

impl DataPreparer for SHA256DataPreparer {
    #[allow(non_snake_case)]
    fn prepare(&self, data: &Vec<u8>) -> DataPrepareResult {
        let length = data.len();
        let mut vars: [u32; 8] = [
            0x6A09E667,
            0xBB67AE85,
            0x3C6EF372,
            0xA54FF53A,
            0x510E527F,
            0x9B05688C,
            0x1F83D9AB,
            0x5BE0CD19
        ];
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

        if length < 64 {
            let data_remains = data.clone();
            let mut service_data = Vec::new();
            for i in 0..vars.len() {
                let buff: [u8; 4] = vars[i].to_be_bytes();
                for j in 0..buff.len() {
                    service_data.push(buff[j]);
                }
            }
            return DataPrepareResult { data_remains, service_data };
        }

        let parts_amount = length / 64;
        let work_length = parts_amount * 64;
        for chunk in (&data[..work_length]).chunks(64) {
            //only work if u32 length is 4 bytes
            let chunk: &[u8; 64] = <&[u8; 64]>::try_from(chunk).unwrap();
            unsafe {
                let words: &mut [u32; 64] = &mut [0; 64];
                for i in 0..16 {
                    let buff: &[u8; 4] = <&[u8; 4]>::try_from(&chunk[i * 4..i * 4 + 4]).unwrap();
                    words[i] = mem::transmute_copy(buff);
                }
                for i in 16..64 {
                    let s0 = words[i - 15].clone().rotate_right(7)
                        ^ words[i - 15].clone().rotate_right(18)
                        ^ words[i - 15].clone().rotate_right(3);
                    let s1 = words[i - 2].clone().rotate_right(17)
                        ^ words[i - 2].clone().rotate_right(19)
                        ^ words[i - 2].clone().rotate_right(10);
                    words[i] = words[i - 16].clone()
                        .overflowing_add(s0).0
                        .overflowing_add(words[i - 7].clone()).0
                        .overflowing_add(s1).0;
                }
                let mut buff = vars.clone();
                for i in 0..64 {
                    let sum0: u32 = buff[0].clone().rotate_right(2)
                        ^ buff[0].clone().rotate_right(13)
                        ^ buff[0].clone().rotate_right(22);
                    let Ma: u32 = (buff[0] & buff[1])
                        ^ (buff[0] & buff[2])
                        ^ (buff[1] & buff[2]);
                    let t2 = sum0.overflowing_add(Ma).0;
                    let sum1: u32 = words[4].clone().rotate_right(6)
                        ^ words[4].clone().rotate_right(11)
                        ^ words[4].clone().rotate_right(25);
                    let Ch: u32 = (buff[4] & buff[5])
                        ^ ((words[4] ^ u32::MAX) & words[6]);
                    let t1: u32 = words[7].clone()
                        .overflowing_add(sum1).0
                        .overflowing_add(Ch).0
                        .overflowing_add(consts[i]).0
                        .overflowing_add(words[i]).0;

                    buff[7] = buff[6];
                    buff[6] = buff[5];
                    buff[5] = buff[4];
                    buff[4] = buff[3].overflowing_add(t1.clone()).0;
                    buff[3] = buff[2];
                    buff[2] = buff[1];
                    buff[1] = buff[0];
                    buff[0] = t1.overflowing_add(t2).0;
                }
                vars[0] = vars[0].overflowing_add(buff[0]).0;
                vars[1] = vars[1].overflowing_add(buff[1]).0;
                vars[2] = vars[2].overflowing_add(buff[2]).0;
                vars[3] = vars[3].overflowing_add(buff[3]).0;
                vars[4] = vars[4].overflowing_add(buff[4]).0;
                vars[5] = vars[5].overflowing_add(buff[5]).0;
                vars[6] = vars[6].overflowing_add(buff[6]).0;
                vars[7] = vars[7].overflowing_add(buff[7]).0;
            }
        }
        let data_remains = Vec::from(&data[work_length..]);
        let mut service_data = Vec::new();
        for i in 0..vars.len() {
            let buff: [u8; 4] = vars[i].to_be_bytes();
            for j in 0..buff.len() {
                service_data.push(buff[j]);
            }
        }
        return DataPrepareResult { data_remains, service_data };

        //unimplemented!()
    }
}
