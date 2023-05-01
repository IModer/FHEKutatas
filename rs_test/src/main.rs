#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
use std::time::Instant;
use rand::Rng;
use tfhe::integer::{ServerKey, gen_keys_radix, ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use rayon::{join};

pub mod third_impl;
pub mod high_api_impl;
pub mod first_impl;

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

const MAXLISTLENGTH : usize = 500;
const MAXVALUE : u16 = 100;


fn main() {
    // We generate a set of client/server keys, using the default parameters:
    let num_block = 4;
    let size = 500;
    //let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_block);

    // Define varibles, this should be random
    let mut rng = rand::thread_rng();

    let mut s_clear : Vec<u16> = vec![0; size];
    let mut b_clear : Vec<u16> = vec![0; size];

    
    let mut s_clear64 : Vec<u64> = vec![0; size];
    let mut b_clear64 : Vec<u64> = vec![0; size];
    
    for x in &mut s_clear {
        *x = rng.gen_range(0..MAXVALUE);
    }

    for x in &mut b_clear {
        *x = rng.gen_range(0..MAXVALUE);
    }

    for x in &mut s_clear64 {
        *x = rng.gen_range(0..MAXVALUE as u64);
    }

    for x in &mut b_clear64 {
        *x = rng.gen_range(0..MAXVALUE as u64);
    }
    // //let clear_a1 = 10u64; let clear_a2 = 10u64;
    // //let clear_b1 = 11u64; let clear_b2 = 10u64;
    
    println!("Input : \n s = {s_clear64:?} \n b = {b_clear64:?}");

    // Encrypt values
    
    // This i dont know about
    // let mut s = client_key.encrypt(s);
    // let mut b = client_key.encrypt(b);
    /*let mut s = Vec::with_capacity(size);
    let mut b = Vec::with_capacity(size);  // It can be vec![] ? Might be difficult

    for i in 0..size {
        s.push(client_key.encrypt(s_clear[i]));
        b.push(client_key.encrypt(b_clear[i]));
    }*/

    // //let mut a1 = client_key.encrypt(clear_a1);
    // //let mut a2 = client_key.encrypt(clear_a2);
    // //let mut b1 = client_key.encrypt(clear_b1);
    // //let mut b2 = client_key.encrypt(clear_b2);

    
    // Start of timer
    let now = Instant::now();

    // Call to algo

    third_impl::setup(&mut s_clear64, &mut b_clear64, num_block);
    //high_api_impl::setup(&mut s_clear, &mut b_clear, num_block);

    //End of timer
    let elapsed = now.elapsed();

    /*for _i in 0..10 {
        b1 = server_key.min_parallelized(&mut a1, &mut a2);
    }*/
    
    /*for _i in 0..10 {
        add(&mut a1, &mut a2, &mut b1, &server_key);
        sub(&mut a1, &mut a2, &mut b1, &server_key);
        
        b1 = min(&mut a1, &mut a2, &mut b1, &server_key);
    }
    (b1, b2) = min2(&mut a1, &mut a2, &mut b1, &mut b2, &server_key);*/
    
    // Decrypt results retrieve from the server

    //type DebugT = BaseRadixCiphertext<CiphertextBase<BaseRadixCiphertext<CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>>>;

    /*for i in 0..MAXLISTLENGTH {
        s_clear[i] = client_key.decrypt(&s[i]);
        b_clear[i] = client_key.decrypt(&b[i]);
        //println!("Debug = {} {}", s_clear[i], b_clear[i]);
    }*/

    // // let result_a1: u64 = client_key.decrypt(&a1);
    // // let result_a2: u64 = client_key.decrypt(&a2);
    // // let result_b1: u64 = client_key.decrypt(&b1);
    // // let result_b2: u64 = client_key.decrypt(&b2);
    // // let result_a = result_a1 + result_a2*256;
    // // let result_b = result_b1 + result_b2*256;

    // Print results

    println!("Result : \n s = {s_clear64:?} \n b = {b_clear64:?}");
    println!("Times elapsed {elapsed:.2?}");

    // //println!("{result_a}, {result_b}");
    // //println!("Elapsed: {elapsed:.2?}");
}
