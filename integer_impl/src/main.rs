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

fn main()
{
    // We generate a set of client/server keys, using the default parameters:
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, NUM_BLOCK);

    // Define varibles, this should be random
    let mut rng = rand::thread_rng();

    let mut s_clear : Vec<u64> = vec![0; MAXLISTLENGTH];
    let mut b_clear : Vec<u64> = vec![0; MAXLISTLENGTH];
    
    for x in &mut s_clear {
        *x = rng.gen_range(0..MAXVALUE);
    }

    for x in &mut b_clear {
        *x = rng.gen_range(0..MAXVALUE);
    }
    
    println!("Input : \n s = {s_clear:?} \n b = {b_clear:?}");
    // Encrypt values
    
    // This i dont know about
    // let mut s = client_key.encrypt(s);
    // let mut b = client_key.encrypt(b);
    let mut s = Vec::with_capacity(MAXLISTLENGTH);
    let mut b = Vec::with_capacity(MAXLISTLENGTH);  // It can be vec![] ? Might be difficult
    
    for i in 0..MAXLISTLENGTH {
        s.push(client_key.encrypt(s_clear[i]));
        b.push(client_key.encrypt(b_clear[i]));
    }
    

    // Start of timer
    let now = Instant::now();

    // Call to algo

    volume_match(&mut s, &mut b, &server_key);

    //End of timer
    let elapsed = now.elapsed();


    for i in 1..MAXLISTLENGTH {
        s_clear[i] = client_key.decrypt(&s[i]);
        b_clear[i] = client_key.decrypt(&b[i]);
    }

    // Print results

    println!("Result : \n s = {s_clear:?} \n b = {b_clear:?}");
    println!("Times elapsed {elapsed:.2?}");

}