#![allow(non_snake_case)]
use tfhe::integer::{ServerKey, gen_keys_radix,  ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap, parameters::PARAM_MESSAGE_2_CARRY_2};
use rayon::{prelude::*, join};
use std::time::{Instant, Duration};
use crate::logging;

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

pub fn run(s_clear: &mut Vec<u64>, b_clear: &mut Vec<u64>, NUM_BLOCK: usize) {
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, NUM_BLOCK);
    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(client_key.encrypt(s_clear[i]));
    }

    for i in 0..s_clear.len() {
        b.push(client_key.encrypt(b_clear[i]));
    }

    let now = Instant::now();
    
    volume_match(&mut s, &mut b, NUM_BLOCK, &server_key);

    let elapsed: Duration = now.elapsed();
    logging::log("full_paral total", elapsed);

    for i in 0..s.len() {
        s_clear[i] = client_key.decrypt(&s[i]);
    }
    for i in 0..b.len() {
        b_clear[i] = client_key.decrypt(&b[i]);
    }
}

pub fn volume_match(
    s : &mut Vec<Ciphertext>,
    b : &mut Vec<Ciphertext>,
    NUM_BLOCK: usize,
    server_key: &ServerKey,
) {
    // Init variables
    let mut S: Vec<Ciphertext>  = Vec::with_capacity(s.len()+1);
    let mut B: Vec<Ciphertext>  = Vec::with_capacity(b.len()+1);
    
    for _ in 0..s.len()+1 {
        S.push(server_key.create_trivial_zero_radix(NUM_BLOCK));
    }
    for _ in 0..b.len()+1 {
        B.push(server_key.create_trivial_zero_radix(NUM_BLOCK));
    }

    // Sum into S and B in paralell
    //let now = Instant::now();

    join(
        || (for i in 0..s.len() {S[i+1] = add(&mut S[i], &mut s[i], server_key);}), 
        || (for i in 0..b.len() {B[i+1] = add(&mut B[i],&mut b[i], server_key);})
    );

    //let elapsed = now.elapsed();
    //logging::log("full_paral summing", elapsed);
    
    // Min of S and B
    //let now = Instant::now();
    
    let mut L = server_key.smart_min_parallelized(&mut S[s.len()], &mut B[b.len()]);
    server_key.full_propagate_parallelized(&mut L);

    //let elapsed = now.elapsed();
    //logging::log("full_paral setting up the transaction volume", elapsed);
    
    // Calculate new s and b <- Parallalise this
    //let now = Instant::now();
    join(
        ||{
            *s = (&mut s.clone(), &mut S).into_par_iter()
                .map(|(x, y)| calc_order(x, y, &L, &server_key))
                .collect();
        },
        ||{
            *b = (&mut b.clone(), &mut B).into_par_iter()
                .map(|(x, y)| calc_order(x, y, &L, &server_key))
                .collect();
        }
    );
    //let elapsed = now.elapsed();
    
    //logging::log("full_paral loop", elapsed);
}

fn add(
    a: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey
) -> Ciphertext {
    if !server_key.is_add_possible(a, &b) {
        server_key.full_propagate_parallelized(a);
    }
    server_key.unchecked_add(a, &b)
}

fn calc_order(
    x16: &mut Ciphertext,
    X: &mut Ciphertext,
    L: &Ciphertext,
    server_key: &ServerKey,
) -> Ciphertext {
    server_key.full_propagate_parallelized(X);
    let mut t:Ciphertext = server_key.unchecked_min_parallelized(L, X);
    
    server_key.full_propagate_parallelized(&mut t);
    let mut m: Ciphertext = server_key.unchecked_sub(L, &t);
    
    server_key.full_propagate_parallelized(&mut m);
    server_key.unchecked_min_parallelized(x16, &m)
}