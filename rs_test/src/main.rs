#![allow(non_snake_case)]
use std::time::Instant;
use rand::Rng;
use tfhe::integer::RadixClientKey;
use tfhe::integer::{ServerKey, gen_keys_radix, ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use rayon::{join};

type Cipertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

const MAXLISTLENGTH : usize = 10;  //500
const MAXVALUE : u64 = 10;

/*fn add(
    a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    server_key: &ServerKey) {
        server_key.unchecked_add_assign(a1, b);

        let mut carry = server_key.key.carry_extract(&a1.blocks[3]);

        server_key.smart_add_assign_parallelized(a2, &mut carry);

        server_key.full_propagate_parallelized(a1);
    }*/

fn add(
    a1: &mut Cipertext,
    a2: &mut Cipertext,
    b: &mut Cipertext,
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
    a1: &mut Cipertext,
    a2: &mut Cipertext,
    b: &mut Cipertext,
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
    a1: &mut Cipertext,
    a2: &mut Cipertext,
    b: &mut Cipertext,
    server_key: &ServerKey)
    -> Cipertext {
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
    a1: &mut Cipertext,
    a2: &mut Cipertext,
    b1: &mut Cipertext,
    b2: &mut Cipertext,
    server_key: &ServerKey)
    -> (Cipertext,
        Cipertext)  {
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
    s : &mut Vec<Cipertext>,
    b : &mut Vec<Cipertext>,
    server_key: &ServerKey,
    //client_key : &RadixClientKey
)
{
    let size = 10;

    // Init variables

    let mut S_1  = server_key.create_trivial_zero_radix(4);
    let mut S_2  = server_key.create_trivial_zero_radix(4);
    let mut B_1  = server_key.create_trivial_zero_radix(4);
    let mut B_2  = server_key.create_trivial_zero_radix(4);

    // Sum into S and B in paralell

    join(
        || (for i in 0..size {add(&mut S_1, &mut S_2, &mut s[i], server_key);}), 
        || (for i in 0..size {add(&mut B_1, &mut B_2, &mut b[i], server_key);})
    );

    // for i in 0..size {
    //     add(&mut S_1, &mut S_2, &mut s[i], server_key);
    // }

    // for i in 0..size {
    //     add(&mut B_1, &mut B_2, &mut b[i], server_key);
    // }

    // Min of S and B

    let (mut lTT_1 , mut lTT_2) = min2(&mut S_1,&mut S_2,&mut B_1,&mut B_2, server_key);
    
    // let S_1clear : u64 = client_key.decrypt(&S_1);
    // let S_2clear : u64 = client_key.decrypt(&S_2);
    // let B_1clear : u64 = client_key.decrypt(&B_1);
    // let B_2clear : u64 = client_key.decrypt(&B_2);
    
    // print!("Debug : \nS : {} {} \nB : {} {}\n", S_1clear, S_2clear, B_1clear, B_2clear);


    
    // Calculate new s and b <- Parallalise this
    
    let (mut lTT_1c, mut lTT_2c) = (lTT_1.clone(), lTT_2.clone());  //This might actually outweight the gains of paralellism
    
    // let lTT_1clear : u64 = client_key.decrypt(&lTT_1);
    // let lTT_2clear : u64 = client_key.decrypt(&lTT_2);

    // let lTT_1clearc : u64 = client_key.decrypt(&lTT_1c);
    // let lTT_2clearc : u64 = client_key.decrypt(&lTT_2c);

    // println!("Debug : lTT {} {}", lTT_1clear, lTT_2clearc);
    // println!("Debug : lTTc {} {}", lTT_1clear, lTT_2clearc);


    join(
        ||(for i in 0..size
                    {
                        s[i] = min(&mut S_1,&mut S_2, &mut s[i], server_key);
                        sub(&mut lTT_1 , &mut lTT_2, &mut s[i], server_key);
                    }),
        ||(for i in 0..size
                    {
                        b[i] = min(&mut B_1,&mut B_2, &mut b[i], server_key);
                        sub(&mut lTT_1c , &mut lTT_2c, &mut b[i], server_key);
                    })
    );

    // for i in 0..size {
    //     s[i] = min(&mut S_1,&mut S_2, &mut s[i], server_key);
    //     // let t : u64 = client_key.decrypt(&s[i]);
    //     // println!("Debug: s[{i}] : {}", t);
    //     sub(&mut lTT_1 , &mut lTT_2, &mut s[i], server_key);
    //     // let tt1 : u64 = client_key.decrypt(&lTT_1);
    //     // let tt2 : u64 = client_key.decrypt(&lTT_2);
    //     // println!("Debug: tt at {i} : {} {}", tt1, tt2);
    // }

    // let (mut lTT_1 , mut lTT_2) = min2(&mut S_1,&mut S_2,&mut B_1,&mut B_2, server_key);

    // for i in 0..size {
    //     b[i] = min(&mut B_1,&mut B_2, &mut b[i], server_key);
    //     sub(&mut lTT_1 , &mut lTT_2, &mut b[i], server_key);
    // }
    
}

fn main() {
    // We generate a set of client/server keys, using the default parameters:
    let num_block = 4;
    let size = 10;
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_block);

    // Define varibles, this should be random
    let mut rng = rand::thread_rng();

    let mut s_clear : Vec<u64> = vec![0; size];
    let mut b_clear : Vec<u64> = vec![0; size];
    
    for x in &mut s_clear {
        *x = rng.gen_range(0..MAXVALUE);
    }

    for x in &mut b_clear {
        *x = rng.gen_range(0..MAXVALUE);
    }
    // //let clear_a1 = 10u64; let clear_a2 = 10u64;
    // //let clear_b1 = 11u64; let clear_b2 = 10u64;
    
    println!("Input : \n s = {s_clear:?} \n b = {b_clear:?}");

    // Encrypt values
    
    // This i dont know about
    // let mut s = client_key.encrypt(s);
    // let mut b = client_key.encrypt(b);
    let mut s = Vec::with_capacity(size);
    let mut b = Vec::with_capacity(size);  // It can be vec![] ? Might be difficult

    for i in 0..size {
        s.push(client_key.encrypt(s_clear[i]));
        b.push(client_key.encrypt(b_clear[i]));
    }

    // //let mut a1 = client_key.encrypt(clear_a1);
    // //let mut a2 = client_key.encrypt(clear_a2);
    // //let mut b1 = client_key.encrypt(clear_b1);
    // //let mut b2 = client_key.encrypt(clear_b2);


    
    // Start of timer
    let now = Instant::now();

    // Call to algo

    volume_match(&mut s, &mut b, &server_key);

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

    for i in 0..MAXLISTLENGTH {
        s_clear[i] = client_key.decrypt(&s[i]);
        b_clear[i] = client_key.decrypt(&b[i]);
        //println!("Debug = {} {}", s_clear[i], b_clear[i]);
    }

    // // let result_a1: u64 = client_key.decrypt(&a1);
    // // let result_a2: u64 = client_key.decrypt(&a2);
    // // let result_b1: u64 = client_key.decrypt(&b1);
    // // let result_b2: u64 = client_key.decrypt(&b2);
    // // let result_a = result_a1 + result_a2*256;
    // // let result_b = result_b1 + result_b2*256;

    // Print results

    println!("Result : \n s = {s_clear:?} \n b = {b_clear:?}");
    println!("Times elapsed {elapsed:.2?}");

    // //println!("{result_a}, {result_b}");
    // //println!("Elapsed: {elapsed:.2?}");
}
