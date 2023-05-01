use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint16};
use tfhe::prelude::*;
use std::time::Instant;
use rand::Rng;

pub fn setup(s_clear: &mut Vec<u16>, b_clear: &mut Vec<u16>, _NUM_BLOCK: usize) {
    let config = ConfigBuilder::all_disabled()
        .enable_default_uint16()
        .build();

    // Client-side
    let (client_key, server_key) = generate_keys(config);

    set_server_key(server_key);


    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(FheUint16::encrypt(s_clear[i], &client_key));
    }

    for i in 0..s_clear.len() {
        b.push(FheUint16::encrypt(b_clear[i], &client_key));
    }

    let now = Instant::now();

    volume_match(&mut s, &mut b);

    let elapsed = now.elapsed();
    println!("Time for the high api implementation: {elapsed:.2?}");

    for i in 0..s_clear.len()
    {
        s_clear[i] = s[i].decrypt(&client_key);
    }

    for i in 0..b_clear.len()
    {
        b_clear[i] = b[i].decrypt(&client_key);
    }

}


fn volume_match(s: &mut Vec<FheUint16>, b: &mut Vec<FheUint16>){
    let mut sell_vol = FheUint16::encrypt_trivial(0u16);
    let mut buy_vol = FheUint16::encrypt_trivial(0u16);

    //Sum s and b

    for i in 0..s.len() {
        sell_vol = sell_vol + &s[i];
    }
    for i in 0..b.len() {
        buy_vol = buy_vol + &b[i];
    }

    //S functions now as the first leftvol/transvol B as the second
    sell_vol = sell_vol.min(&buy_vol);
    buy_vol = sell_vol.clone();


    for i in 0..s.len() {
        s[i] = s[i].min(&sell_vol);
        sell_vol = sell_vol - &s[i];
    }

    for i in 0..b.len() {
        b[i] = b[i].min(&buy_vol);
        buy_vol = buy_vol - &b[i];
    }
}