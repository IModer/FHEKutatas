use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint16};
use tfhe::prelude::*;
use std::time::Instant;

fn main() {
    let config = ConfigBuilder::all_disabled()
        .enable_default_uint16()
        .build();

    // Client-side
    let (client_key, server_key) = generate_keys(config);

    let clear_a = 27u16;
    let clear_b = 30u16;

    let mut a = FheUint16::encrypt(clear_a, &client_key);
    let mut b = FheUint16::encrypt(clear_b, &client_key);

    //Server-side
    set_server_key(server_key);

    let now = Instant::now();

    for _i in 0..10 {
        a = a.min(&b);
    }
    let elapsed = now.elapsed();

    //Client-side
    let decrypted_result: u16 = a.decrypt(&client_key);
    let decrypted_result_b: u16 = b.decrypt(&client_key);

    let clear_result = clear_a + clear_b;

    println!("{}, {}", decrypted_result, decrypted_result_b);
    println!("Elapsed: {:.2?}", elapsed);
}