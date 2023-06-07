#![allow(non_snake_case)]
use tfhe::integer::RadixClientKey;
use tfhe::integer::{ServerKey, gen_keys_radix,  ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use rayon::{prelude::*, join};
use tfhe::integer::ciphertext::IntegerCiphertext;
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use std::time::{Instant, Duration};
use crate::logging;

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

pub fn run(s_clear: &mut Vec<u64>, b_clear: &mut Vec<u64>, _NUM_BLOCK: usize) {
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, _NUM_BLOCK);
    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    let mut s16 = Vec::with_capacity(s_clear.len());
    let mut b16 = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(client_key.encrypt(s_clear[i]));
        s16.push(fromNto2Nbit(&s[i], &server_key));
    }

    for i in 0..s_clear.len() {
        b.push(client_key.encrypt(b_clear[i]));
        b16.push(fromNto2Nbit(&b[i], &server_key));
    }

    let now = Instant::now();
    
    volume_match(&mut s, &mut b, &mut s16, &mut b16, _NUM_BLOCK, &server_key, &client_key);

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
    s16 : &mut Vec<Ciphertext>,
    b16 : &mut Vec<Ciphertext>,
    NUM_BLOCK: usize,
    server_key: &ServerKey,
    client_key: &RadixClientKey
) {
    // Init variables
    let mut S: Vec<Ciphertext>  = Vec::with_capacity(s.len()+1);
    let mut B: Vec<Ciphertext>  = Vec::with_capacity(b.len()+1);
    
    for i in 0..s.len()+1 {
        S.push(server_key.create_trivial_zero_radix(NUM_BLOCK*2));
    }
    for i in 0..b.len()+1 {
        B.push(server_key.create_trivial_zero_radix(NUM_BLOCK*2));
    }

    // Sum into S and B in paralell
    //let now = Instant::now();

    join(
        || (for i in 0..s.len() {S[i+1] = add_assign(&mut S[i], &mut s[i], server_key);}), 
        || (for i in 0..b.len() {B[i+1] = add_assign(&mut B[i],&mut b[i], server_key);})
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
            *s = (s16, &mut S).into_par_iter()
                .map(|(x, y)| min(x, y, &L, &server_key))
                .collect();
        },
        ||{
            *b = (b16, &mut B).into_par_iter()
                .map(|(x, y)| min(x, y, &L, &server_key))
                .collect();
        }
    );
    //let elapsed = now.elapsed();
    
    //logging::log("full_paral loop", elapsed);
}

fn fromNto2Nbit(x: &Ciphertext, server_key: &ServerKey) -> Ciphertext
{
    let NUM_BLOCK: usize = x.blocks().len();
    let temp: Ciphertext = server_key.create_trivial_zero_radix(NUM_BLOCK);

    let mut u = Vec::new();
    u.extend_from_slice(x.blocks());
    u.extend_from_slice(temp.blocks());

    IntegerCiphertext::from_blocks(u)
}

fn from2NtoNbit(x: &mut Ciphertext) -> Ciphertext
{
    IntegerCiphertext::from_blocks(x.blocks()[0..x.blocks().len()/2].to_vec())
}

fn add_assign(
    a: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey
) -> Ciphertext {
    if !server_key.is_add_possible(a, &b) {
        server_key.full_propagate_parallelized(a);
    }
    server_key.unchecked_add(a, &b)
}

fn min(
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
    *x16 = server_key.unchecked_min_parallelized(x16, &m);

    from2NtoNbit(x16)
}