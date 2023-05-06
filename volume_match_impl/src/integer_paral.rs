use std::time::{Instant, Duration};
use tfhe::integer::{ServerKey, gen_keys_radix, ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use rayon::{join};
use crate::logging;

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

// TODO: write this
pub fn run(s_clear: &mut Vec<u64>, b_clear: &mut Vec<u64>, _NUM_BLOCK: usize, size : usize) {

    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, _NUM_BLOCK);
    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(client_key.encrypt(s_clear[i]));
    }

    for i in 0..s_clear.len() {
        b.push(client_key.encrypt(b_clear[i]));
    }

    let now = Instant::now();
    println!("----------------------\nRunning integer_paral");

    volume_match(&mut s, &mut b, &server_key, size);

    let elapsed = now.elapsed();
    println!("Time for the integer_paral: {elapsed:.2?}\n----------------------");

    for i in 0..s.len() {
        s_clear[i] = client_key.decrypt(&s[i]);
    }
    for i in 0..b.len() {
        b_clear[i] = client_key.decrypt(&b[i]);
    }
}

fn add(
    a1: &mut Ciphertext,
    a2: &mut Ciphertext,
    b: & Ciphertext,
    server_key: &ServerKey) {
        let mut s = a1.clone(); 

        if !server_key.is_add_possible(a1, b) {
            //let carry = server_key.key.carry_extract(a1.blocks());
            server_key.full_propagate_parallelized(a1);
        }
        server_key.unchecked_add_assign(a1, b);
        
        let mut z1 = server_key.smart_gt_parallelized(&mut s,a1);

        if !server_key.is_add_possible(a2, &mut z1) {
            server_key.full_propagate_parallelized(a2);
        }
        server_key.unchecked_add_assign(a2, &mut z1);

    }

fn sub(
    a1: &mut Ciphertext,
    a2: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey) {
        let mut s = a1.clone(); 

        
        if !server_key.is_sub_possible(a1, b) {
            server_key.full_propagate_parallelized(a1);
        }
        server_key.unchecked_sub_assign(a1, b);
        

        let mut z1 = server_key.smart_lt_parallelized(&mut s,a1);

        if !server_key.is_sub_possible(a2, &mut z1) {
            server_key.full_propagate_parallelized(a2);
        }
        server_key.unchecked_sub_assign(a2, &mut z1);
    }

fn min(
    a1: &mut Ciphertext,
    a2: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey)
    -> Ciphertext {
        let mut z1 = server_key.smart_gt_parallelized(a2, &mut server_key.create_trivial_zero_radix(4));
        let mut z2 = server_key.smart_gt_parallelized(a1, b);

        //Bitand or multiplication both work
        // (b-a1)*(z1*z2) + a1
        server_key.smart_bitor_assign_parallelized(&mut z1, &mut z2);
        
        if !server_key.is_sub_possible(b, a1) {
            server_key.full_propagate_parallelized(a1);
        }
        let mut d = server_key.unchecked_sub(b, a1);
        let mut e = server_key.smart_mul_parallelized(&mut d, &mut z1);


        if !server_key.is_add_possible(&mut e, a1) {
            server_key.full_propagate_parallelized(a1);
        }

        server_key.unchecked_add(&mut e, a1)
    }

fn min2(
    a1: &mut Ciphertext,
    a2: &mut Ciphertext,
    b1: &mut Ciphertext,
    b2: &mut Ciphertext,
    server_key: &ServerKey
    ) -> (Ciphertext,Ciphertext)  {
        //Which will be greater?
        // (a2 > b2) || (a2 == b2 && a1 > b1)
        //    z1            z2          z3
        //If this is true, a is bigger
        let mut z1 = server_key.smart_gt_parallelized(a2, b2);
        let mut z2 = server_key.smart_eq_parallelized(a2, b2);
        let mut z3 = server_key.smart_gt_parallelized(a1, b1);

        server_key.smart_bitand_assign_parallelized(&mut z2, &mut z3);
        server_key.smart_bitor_assign_parallelized(&mut z1, &mut z2);
        
        //return c1 c2 will be:
        //c1 = (b1 - a1) * z1 + a1
        //c2 = (b2 - a2) * z1 + a2
        
        //d1 = b1 - a1
        let mut d1 = server_key.smart_sub_parallelized(b1, a1);
        //e1 = s1 * z1 = (b1 - a1) * z1
        let mut e1 = server_key.smart_mul_parallelized(&mut d1, &mut z1);

        //c1 = e1 + b1 = (b1 - a1) * z1 + a1
        if !server_key.is_add_possible(&mut e1, a1) {
            server_key.full_propagate_parallelized(a1);
        }
        let c1 = server_key.unchecked_add(&mut e1, a1);

        //d2 = b2 - a2
        let mut d2 = server_key.smart_sub_parallelized(b2, a2);
        //e2 = d2 * z1 = (b2 - a2) * z1
        let mut e2 = server_key.smart_mul_parallelized(&mut d2, &mut z1);

        //c2 = e2 + b2 = (b2 - a2) * z1 + a2
        if !server_key.is_add_possible(&mut e2, a2) {
            server_key.full_propagate_parallelized(a2);
        }
        let c2 = server_key.unchecked_add(&mut e2, a2);

        return (c1, c2);
    }

fn volume_match(
    s : &mut Vec<Ciphertext>,
    b : &mut Vec<Ciphertext>,
    server_key: &ServerKey,
    size : usize
) {

    // Init variables

    let mut S_1  = server_key.create_trivial_zero_radix(4);
    let mut S_2  = server_key.create_trivial_zero_radix(4);
    let mut B_1  = server_key.create_trivial_zero_radix(4);
    let mut B_2  = server_key.create_trivial_zero_radix(4);

    // Sum into S and B in paralell
    let now = Instant::now();

    join(
        || (for i in 0..size {add(&mut S_1, &mut S_2, &mut s[i], server_key);}), 
        || (for i in 0..size {add(&mut B_1, &mut B_2, &mut b[i], server_key);})
    );

    let elapsed = now.elapsed();
    println!("integer_paral : Summing s and b: {elapsed:.2?}");
    

    // Min of S and B
    let now = Instant::now();

    let (mut lTT_1 , mut lTT_2) = min2(&mut S_1,&mut S_2,&mut B_1,&mut B_2, server_key);
    let (mut lTT_1c, mut lTT_2c) = (lTT_1.clone(), lTT_2.clone());  //This might actually outweight the gains of paralellism
    
    let elapsed = now.elapsed();
    println!("integer_paral : Setting up leftvols: {elapsed:.2?}");

    // Calculate new s and b <- Parallalise this
    
    let mut min_dur = Duration::new(0,0);
    let mut sub_dur = Duration::new(0,0);

    join(
        ||(for i in 0..size
                    {
                        let now2 = Instant::now();

                        s[i] = min(&mut lTT_1,&mut lTT_2, &mut s[i], server_key);
                        
                        min_dur += now2.elapsed();
                        let now2 = Instant::now();
                        
                        sub(&mut lTT_1 , &mut lTT_2, &mut s[i], server_key);
                    
                        sub_dur += now2.elapsed();
                    }),
        ||(for i in 0..size
                    {
                        b[i] = min(&mut lTT_1c,&mut lTT_2c, &mut b[i], server_key);
                        sub(&mut lTT_1c , &mut lTT_2c, &mut b[i], server_key);
                    })
    );
    
    let elapsed = now.elapsed();
    
    println!("integer_paral : Subtracting only s: {sub_dur:.2?}");
    println!("integer_paral : Min only s: {min_dur:.2?}");
    println!("integer_paral : Subtracting and min: {elapsed:.2?}");


}