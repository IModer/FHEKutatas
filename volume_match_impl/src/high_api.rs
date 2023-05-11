use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint16};
use tfhe::prelude::*;
use std::time::{Instant, Duration};
use crate::logging;

pub fn run(s_clear: &mut Vec<u16>, b_clear: &mut Vec<u16>, _NUM_BLOCK: usize) {
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
    //println!("----------------------\nRunning high_api");

    volume_match(&mut s, &mut b);

    let elapsed = now.elapsed();
    
    logging::log("high_api total", elapsed);
    //println!("Time for the high_api : {elapsed:.2?}");

    for i in 0..s_clear.len()
    {
        s_clear[i] = s[i].decrypt(&client_key);
    }

    for i in 0..b_clear.len()
    {
        b_clear[i] = b[i].decrypt(&client_key);
    }

    //println!("Result for high_api : s = {s_clear:?} b = {b_clear:?}\n----------------------");

}


fn volume_match(s: &mut Vec<FheUint16>, b: &mut Vec<FheUint16>){
    let mut sell_vol = FheUint16::encrypt_trivial(0u16);
    let mut buy_vol = FheUint16::encrypt_trivial(0u16);

    //Sum into sell_vol and buy_vol 
    let now = Instant::now();

    for i in 0..s.len() {
        sell_vol = sell_vol + &s[i];
    }
    for i in 0..b.len() {
        buy_vol = buy_vol + &b[i];
    }

    let elapsed = now.elapsed();
    logging::log("high_api summing", elapsed);
    //println!("high_api : Summing s and b: {elapsed:.2?}");
   

    //S functions now as the first leftvol/transvol B as the second
    let now = Instant::now();
    
    sell_vol = sell_vol.min(&buy_vol);
    buy_vol = sell_vol.clone();

    let elapsed = now.elapsed();
    logging::log("high_api leftvols", elapsed);
    //println!("high_api : Setting up leftvols: {elapsed:.2?}");

    //Calculate new s and b

    let mut min_dur = Duration::new(0,0);
    let mut sub_dur = Duration::new(0,0);

    for i in 0..s.len() {
        let now2 = Instant::now();

        s[i] = s[i].min(&sell_vol);

        min_dur += now2.elapsed();
        let now2 = Instant::now();

        sell_vol = sell_vol - &s[i];

        sub_dur += now2.elapsed();
    }

    for i in 0..b.len() {
        b[i] = b[i].min(&buy_vol);
        buy_vol = buy_vol - &b[i];
    }

    let elapsed = now.elapsed();
    
    //println!("high_api : Subtracting only s: {sub_dur:.2?}");
    //println!("high_api : Min only s: {min_dur:.2?}");
    //println!("high_api : Subtracting and min: {elapsed:.2?}");
    logging::log("high_api loop", elapsed);

}