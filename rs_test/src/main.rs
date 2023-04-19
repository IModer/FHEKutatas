use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};
use tfhe::prelude::*;

fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let config = ConfigBuilder::all_disabled()
        .enable_default_uint8()
        .enable_default_uint16()
        .build();

    let (client_key, server_key) = generate_keys(config);

    set_server_key(server_key);

    let clear_a = 27u8;
    let clear_b = 122u8;

    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);

    let result = a * b;

    let decrypted_result: u8 = result.decrypt(&client_key);

    let clear_result = clear_a * clear_b;

    assert_eq!(decrypted_result, clear_result);
    println!("{}", clear_result);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}