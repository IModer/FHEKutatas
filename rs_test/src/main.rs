use std::time::Instant;
use tfhe::integer::ServerKey;
use tfhe::integer::gen_keys_radix;
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;

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
    a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    server_key: &ServerKey) {
        let mut s = a1.clone(); 

        
        if !server_key.is_add_possible(a1, b) {
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
    a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
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
    a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    server_key: &ServerKey)
    -> tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>> {
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
    a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    server_key: &ServerKey)
    -> (tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
        tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>)  {
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
        //c1 = (a1-b1) * z1 + b1
        //c2 = (a2-b2) * z1 + b2
        
        let mut d1 = server_key.smart_sub_parallelized(a1, b1);
        let mut e1 = server_key.smart_mul_parallelized(&mut d1, &mut z1);


        if !server_key.is_add_possible(&mut e1, a1) {
            server_key.full_propagate_parallelized(a1);
        }
        let c1 = server_key.unchecked_add(&mut e1, a1);

        
        let mut d2 = server_key.smart_sub_parallelized(a2, b2);
        let mut e2 = server_key.smart_mul_parallelized(&mut d2, &mut z1);


        if !server_key.is_add_possible(&mut e2, a2) {
            server_key.full_propagate_parallelized(a2);
        }
        let c2 = server_key.unchecked_add(&mut e2, a2);

        return (c1, c2);
    }
fn main() {
    // We generate a set of client/server keys, using the default parameters:
    let num_block = 4;
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_block);
    

    let clear_a1 = 10u64;
    let clear_a2 = 10u64;
    let clear_b1 = 11u64;
    let clear_b2 = 10u64;
    
    let mut a1 = client_key.encrypt(clear_a1);
    let mut a2 = client_key.encrypt(clear_a2);
    let mut b1 = client_key.encrypt(clear_b1);
    let mut b2 = client_key.encrypt(clear_b2);
    
    let now = Instant::now();
    
    for _i in 0..10 {
        add(&mut a1, &mut a2, &mut b1, &server_key);
        sub(&mut a1, &mut a2, &mut b1, &server_key);
        
        b1 = min(&mut a1, &mut a2, &mut b1, &server_key);
    }
    (b1, b2) = min2(&mut a1, &mut a2, &mut b1, &mut b2, &server_key);
    
    let elapsed = now.elapsed();

    
    let result_a1: u64 = client_key.decrypt(&a1);
    let result_a2: u64 = client_key.decrypt(&a2);
    let result_b1: u64 = client_key.decrypt(&b1);
    let result_b2: u64 = client_key.decrypt(&b2);
    let result_a = result_a1 + result_a2*256;
    let result_b = result_b1 + result_b2*256;


    println!("{}, {}", result_a, result_b);
    println!("Elapsed: {:.2?}", elapsed);
}
