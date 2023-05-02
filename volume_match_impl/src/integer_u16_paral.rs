#![allow(non_snake_case)]
use std::time::Instant;
use rand::Rng;
use tfhe::integer::{ServerKey, gen_keys_radix, ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use rayon::{join};
type Cipertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

const MAXLISTLENGTH : usize = 10;  //500
const MAXVALUE : u64 = 10;
const NUM_BLOCK: usize = 8;

fn run()
{
    todo!();
}

fn volume_match
    (
    s : &mut Vec<Cipertext>,
    b : &mut Vec<Cipertext>,
    server_key: &ServerKey
    )
{
    // Init variables

    let mut S  = server_key.create_trivial_zero_radix(NUM_BLOCK);
    let mut B  = server_key.create_trivial_zero_radix(NUM_BLOCK);

    // Sum into S and B in paralell

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
    
    // Min of S and B

    S = server_key.smart_min_parallelized(&mut S, &mut B);
    B = S.clone();
    
    // Calculate new s and b <- Parallalise this

    join(
        || (
            for i in 0..MAXLISTLENGTH
            {
                s[i] = server_key.smart_min_parallelized(&mut s[i], &mut S);
                server_key.smart_sub_assign_parallelized(&mut S, &mut s[i]);
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
}