#![allow(non_snake_case)]
use std::time::{Instant, Duration};
use rand::Rng;
use tfhe::integer::{ServerKey, gen_keys_radix, ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use rayon::{join};
type Cipertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

const MAXLISTLENGTH : usize = 10;  //500
const MAXVALUE : u64 = 5;

pub fn run(s_clear: &mut Vec<u64>, b_clear: &mut Vec<u64>, _NUM_BLOCK: usize)
{
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, _NUM_BLOCK*2);
    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(client_key.encrypt(s_clear[i]));
    }

    for i in 0..s_clear.len() {
        b.push(client_key.encrypt(b_clear[i]));
    }

    let now = Instant::now();
    println!("----------------------\nRunning integer_u16_paral");

    volume_match(&mut s, &mut b, &server_key, _NUM_BLOCK);

    let elapsed = now.elapsed();
    println!("Time for the integer_u16_paral: {elapsed:.2?}\n----------------------");

    for i in 0..s.len() {
        s_clear[i] = client_key.decrypt(&s[i]);
    }
    for i in 0..b.len() {
        b_clear[i] = client_key.decrypt(&b[i]);
    }
}

fn volume_match
    (
    s : &mut Vec<Cipertext>,
    b : &mut Vec<Cipertext>,
    server_key: &ServerKey,
    NUM_BLOCK: usize
    )
{
    // Init variables

    let mut S  = server_key.create_trivial_zero_radix(NUM_BLOCK*2);
    let mut B  = server_key.create_trivial_zero_radix(NUM_BLOCK*2);

    // Sum into S and B in paralell

    let now = Instant::now();

    join(
        || (
            for i in 0..MAXLISTLENGTH 
            {
                if !server_key.is_add_possible(&mut S, &mut s[i]) 
                {
                    server_key.full_propagate_parallelized(&mut S);
                }
            server_key.unchecked_add_assign(&mut S, &mut s[i]);
            }
        ), 
        || (
            for i in 0..MAXLISTLENGTH 
            {
                if !server_key.is_add_possible(&mut B, &mut b[i])
                {
                    server_key.full_propagate_parallelized(&mut B);
                }
                server_key.unchecked_add_assign(&mut B, &mut b[i]);
            }
        )
    );
    
    let elapsed = now.elapsed();
    println!("integer_u16_paral : Summing s and b: {elapsed:.2?}");
    // Min of S and B
    
    let now = Instant::now();

    S = server_key.smart_min_parallelized(&mut S, &mut B);
    B = S.clone();
    
    let elapsed = now.elapsed();
    println!("integer_u16_paral : Setting up leftvols: {elapsed:.2?}");
    // Calculate new s and b <- Parallalise this


    
    let mut min_dur = Duration::new(0,0);
    let mut sub_dur = Duration::new(0,0);

    join(
        || (
            for i in 0..MAXLISTLENGTH
            {
                let now2 = Instant::now();

                s[i] = server_key.smart_min_parallelized(&mut s[i], &mut S);

                min_dur += now2.elapsed();

                let now2 = Instant::now();

                server_key.smart_sub_assign_parallelized(&mut S, &mut s[i]);

                sub_dur += now2.elapsed();
            }
        ),
        || (
            for i in 0..MAXLISTLENGTH 
            {
                b[i] = server_key.smart_min_parallelized(&mut b[i], &mut B);
                server_key.smart_sub_assign_parallelized(&mut B, &mut b[i]);
            }
        )
    );

    let elapsed = now.elapsed();
    
    println!("integer_u16_paral : Subtracting only s: {sub_dur:.2?}");
    println!("integer_u16_paral : Min only s: {min_dur:.2?}");
    println!("integer_u16_paral : Subtracting and min: {elapsed:.2?}");
}