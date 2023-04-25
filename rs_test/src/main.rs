use tfhe::integer::ServerKey;
use tfhe::integer::gen_keys_radix;
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;

/*fn add(a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>, server_key: &ServerKey) {
        server_key.unchecked_add_assign(a1, b);

        let mut carry = server_key.key.carry_extract(&a1.blocks[3]);

        server_key.smart_add_assign_parallelized(a2, &mut carry);

        server_key.full_propagate_parallelized(a1);
    }*/

fn add(a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>, server_key: &ServerKey) {
        server_key.unchecked_add_assign(a1, b);


        server_key.smart_add_assign_parallelized(a2, &mut carry);

        server_key.full_propagate_parallelized(a1);
    }

fn min(
    a1: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    a2: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    b: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>,
    server_key: &ServerKey,
    zero: &mut tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>>)
    -> tfhe::integer::ciphertext::BaseRadixCiphertext<tfhe::shortint::CiphertextBase<tfhe::shortint::ciphertext::KeyswitchBootstrap>> {

        //comparison between scalar and encypted value?
        let mut z1 = server_key.smart_gt_parallelized(a2, zero);
        let mut z2 = server_key.smart_gt_parallelized(a1, b);

        //Bitand or multiplication both work
        // (a-b)*(z1*z2) + b
        server_key.smart_bitor_assign_parallelized(&mut z1, &mut z2);
        server_key.smart_add_parallelized(&mut server_key.smart_mul_parallelized(&mut server_key.smart_sub_parallelized(b, a1), &mut z1), a1)
    }
fn main() {
    // We generate a set of client/server keys, using the default parameters:
    let num_block = 4;
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, num_block);
    
    use std::time::Instant;

    let clear_a1 = 100u64;
    let clear_a2 = 3u64;
    let clear_b = 101u64;
    let clear_zero = 0u64;
    
    let mut a1 = client_key.encrypt(clear_a1);
    let mut a2 = client_key.encrypt(clear_a2);
    let mut b = client_key.encrypt(clear_b);
    let mut zero = client_key.encrypt(clear_zero);
    
    let now = Instant::now();

    /*for _i in 0..16 {

        a1 = server_key.smart_min_parallelized(&mut a1, &mut a2);
    }*/

    b = min(&mut a1, &mut a2, &mut b, &server_key, &mut zero);
    let elapsed = now.elapsed();

    
    let result_1: u64 = client_key.decrypt(&a1);
    let result_2: u64 = client_key.decrypt(&a2);
    let result_b: u64 = client_key.decrypt(&b);
    let result = result_1 + result_2*256;


    println!("{}", result_b);
    println!("Elapsed: {:.2?}", elapsed);
}
