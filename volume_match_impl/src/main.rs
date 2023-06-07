#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused)]
use rand::Rng;

use argh::FromArgs;

pub mod integer_padded_paral;
pub mod high_api;
pub mod full_paral;
pub mod integer_u16_paral;
pub mod logging;

#[derive(FromArgs)]
/// Volume match algorithm implemented in TFHE-rs
struct FheArgs {
    /// number of seller/buyers
    #[argh(positional)]
    list_length: usize,

    /// max value a seller or buyer can sell/buy
    #[argh(positional)]
    max_value: u16,
}

fn main() {
    let args: FheArgs = argh::from_env();
    setupAndRun(args.max_value, args.list_length);
}

fn setupAndRun(max_value : u16, list_length : usize)
{
    // We generate a set of client/server keys, using the default parameters:
    const NUM_BLOCK: usize = 4;

    // Define varibles

    //u16 for high_api
    let mut s_clear : Vec<u16> = vec![0; list_length];
    let mut b_clear : Vec<u16> = vec![0; list_length];

    //u16 for integer_u16
    let mut s_clear64_16 : Vec<u64> = vec![0; list_length];
    let mut b_clear64_16 : Vec<u64> = vec![0; list_length];
    
    //u64 for integer_padded 
    let mut s_clear64_p : Vec<u64> = vec![0; list_length];
    let mut b_clear64_p : Vec<u64> = vec![0; list_length];

    //u64 for integer
    let mut s_clear64 : Vec<u64> = vec![0; list_length];
    let mut b_clear64 : Vec<u64> = vec![0; list_length];
    
    //Fill vectors with random values
    //fillWithRandom(&mut s_clear, &mut b_clear);

    fillWithRandomu64(&mut s_clear64, &mut b_clear64, max_value);

    s_clear64_16 = s_clear64.clone();
    b_clear64_16 = b_clear64.clone();

    s_clear64_p = s_clear64.clone();
    b_clear64_p = b_clear64.clone();
    

    //println!("Input: \n s = {s_clear64_16:?} \n b = {b_clear64_16:?}");
    //println!("Input: \n s = {s_clear64:?} \n b = {b_clear64:?}");
    //println!("Input: \n s = {s_clear64_p:?} \n b = {b_clear64_p:?}");
    //println!("Input: \n s = {s_clear64:?} \n b = {b_clear64:?}");

    // Call to algos  //we time inside

    //integer_u16_paral::run(&mut s_clear64_16, &mut b_clear64_16, num_block);
    //high_api::run(&mut s_clear, &mut b_clear, num_block);
    //integer_padded_paral::run(&mut s_clear64_p, &mut b_clear64_p, num_block);
    full_paral::run(&mut s_clear64, &mut b_clear64, 2*NUM_BLOCK);
    //println!("Output: \n s = {s_clear64:?} \n b = {b_clear64:?}");
}

fn fillWithRandomu64(s: &mut Vec<u64>, b: &mut Vec<u64>, max_value : u16) 
{
    let mut rng = rand::thread_rng();

    for x in s {
        *x = rng.gen_range(0..max_value) as u64;
    }
    for x in b {
        *x = rng.gen_range(0..max_value) as u64;
    }
}

fn fillWithRandom(s: &mut Vec<u16>, b: &mut Vec<u16>, max_value : u16) 
{
    let mut rng = rand::thread_rng();

    for x in s {
        *x = rng.gen_range(0..max_value);
    }
    for x in b {
        *x = rng.gen_range(0..max_value);
    }
}
