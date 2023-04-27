use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint16};
use tfhe::prelude::*;
use std::time::Instant;
use rand::Rng;

const MAXVALUE : u16 = 10;


//I don't understand Rust enough to make it work through a function :(
/*fn market(s: &mut Vec<FheUint16>, b: &mut Vec<FheUint16>, mut S: FheUint16, mut B: FheUint16) -> (Vec<FheUint16>,Vec<FheUint16>){
    for i in 1..s.len() {
        S = S + &s[i];
    }
    for i in 1..b.len() {
        B = B + &b[i];
    }

    //S function now as the first leftvol/transvol B as the second

    S = S.min(&B);
    B = S.clone();

    for i in 1..s.len() {
        s[i] = s[i].min(&S);
        S = S - &s[i];
    }
    
    for i in 1..s.len() {
        b[i] = b[i].min(&B);
        B = B - &b[i];
    }

    (s.to_vec(), b.to_vec())
}*/

fn main() {
    let size = 10;
    let config = ConfigBuilder::all_disabled()
        .enable_default_uint16()
        .build();

    // Client-side
    let (client_key, server_key) = generate_keys(config);

    set_server_key(server_key);

    //Set the input sell/buy values
    let mut rng = rand::thread_rng();

    let mut clear_s : Vec<u16> = vec![0; size];
    let mut clear_b : Vec<u16> = vec![0; size];
    
    for x in &mut clear_s {
        *x = rng.gen_range(0..MAXVALUE);
    }

    for x in &mut clear_b {
        *x = rng.gen_range(0..MAXVALUE);
    }

    println!("Input : \n s = {clear_s:?} \n b = {clear_b:?}");
    
    //Encrypt them
    let mut s: Vec<FheUint16> = Vec::new();
    let mut b: Vec<FheUint16> = Vec::new();
    
    for i in 0..clear_s.len()
    {
        s.push(FheUint16::encrypt(clear_s[i], &client_key));
    }
    
    for i in 0..clear_b.len()
    {
        b.push(FheUint16::encrypt(clear_b[i], &client_key));
    }

    let mut sell_vol = FheUint16::encrypt(0u16, &client_key);
    let mut buy_vol = FheUint16::encrypt(0u16, &client_key);


    //Run the algorithm
    let now = Instant::now();

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

    let elapsed = now.elapsed();

    //Decrypt the result
    for i in 0..clear_s.len()
    {
        clear_s[i] = s[i].decrypt(&client_key);
    }

    for i in 0..clear_b.len()
    {
        clear_b[i] = b[i].decrypt(&client_key);
    }

    //Print it
    println!("Output : \n s = {clear_s:?} \n b = {clear_b:?}");

    println!("Elapsed: {:.2?}", elapsed);
}