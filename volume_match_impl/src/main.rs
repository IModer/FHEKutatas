#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
use std::time::Instant;
use rand::Rng;
use tfhe::integer;
use tfhe::integer::{ServerKey, gen_keys_radix, ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use rayon::{join};

pub mod integer_padded_paral;
pub mod high_api;
pub mod integer_paral;
pub mod integer_u16_paral;

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

const MAXLISTLENGTH : usize = 10; //500;
const MAXVALUE : u16 = 5; //100;

fn main() {
    // We generate a set of client/server keys, using the default parameters:
    let num_block = 4;

    // Define varibles

    //u16 for high_api
    let mut s_clear : Vec<u16> = vec![0; MAXLISTLENGTH];
    let mut b_clear : Vec<u16> = vec![0; MAXLISTLENGTH];

    //u16 for integer_u16
    let mut s_clear64_16 : Vec<u64> = vec![0; MAXLISTLENGTH];
    let mut b_clear64_16 : Vec<u64> = vec![0; MAXLISTLENGTH];
    
    //u64 for integer_padded 
    let mut s_clear64_p : Vec<u64> = vec![0; MAXLISTLENGTH];
    let mut b_clear64_p : Vec<u64> = vec![0; MAXLISTLENGTH];

    //u64 for integer
    let mut s_clear64 : Vec<u64> = vec![0; MAXLISTLENGTH];
    let mut b_clear64 : Vec<u64> = vec![0; MAXLISTLENGTH];
    
    //Fill vectors with random values, //this could go in fn
    fillWithRandom(&mut s_clear, &mut b_clear);

    fillWithRandomu64(&mut s_clear64, &mut b_clear64);

    s_clear64_16 = s_clear64.clone();
    b_clear64_16 = b_clear64.clone();

    s_clear64_p = s_clear64.clone();
    b_clear64_p = b_clear64.clone();
    

    //println!("Input for integer_u16_paral : \n s = {s_clear64_16:?} \n b = {b_clear64_16:?}");
    //println!("Input for high_api_paral : \n s = {s_clear64:?} \n b = {b_clear64:?}");
    //println!("Input for integer_padded_paral : \n s = {s_clear64_p:?} \n b = {b_clear64_p:?}");
    //println!("Input for integer_paral : \n s = {s_clear:?} \n b = {b_clear:?}");

    // Call to algos  //we time inside

    integer_u16_paral::run(&mut s_clear64_16, &mut b_clear64_16, num_block);    
    drop(s_clear64_16);drop(b_clear64_16);
    high_api::run(&mut s_clear, &mut b_clear, num_block);
    drop(s_clear);drop(b_clear);
    integer_padded_paral::run(&mut s_clear64_p, &mut b_clear64_p, num_block);
    drop(s_clear64_p);drop(b_clear64_p);
    integer_paral::run(&mut s_clear64, &mut b_clear64, num_block, MAXLISTLENGTH);
    drop(s_clear64);drop(b_clear64);
}


fn fillWithRandomu64(s: &mut Vec<u64>, b: &mut Vec<u64>) 
{
    let mut rng = rand::thread_rng();

    for x in s {
        *x = rng.gen_range(0..MAXVALUE) as u64;
    }
    for x in b {
        *x = rng.gen_range(0..MAXVALUE) as u64;
    }
}

fn fillWithRandom(s: &mut Vec<u16>, b: &mut Vec<u16>) 
{
    let mut rng = rand::thread_rng();

    for x in s {
        *x = rng.gen_range(0..MAXVALUE);
    }
    for x in b {
        *x = rng.gen_range(0..MAXVALUE);
    }
}