#![allow(non_snake_case)]
use tfhe::integer::{ServerKey, gen_keys_radix,  ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use rayon::{join};
use tfhe::integer::ciphertext::IntegerCiphertext;
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use std::time::{Instant, Duration};
use crate::logging;

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

pub fn run(s_clear: &mut Vec<u64>, b_clear: &mut Vec<u64>, _NUM_BLOCK: usize) {
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, _NUM_BLOCK);
    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    let mut s16 = Vec::with_capacity(s_clear.len());
    let mut b16 = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(client_key.encrypt(s_clear[i]));
        s16.push(fromNto2Nbit(&s[i], &server_key));
    }

    for i in 0..s_clear.len() {
        b.push(client_key.encrypt(b_clear[i]));
        b16.push(fromNto2Nbit(&b[i], &server_key));
    }

    let now = Instant::now();
    
    volume_match(&mut s, &mut b, &mut s16, &mut b16, _NUM_BLOCK , &server_key);

    let elapsed: Duration = now.elapsed();
    
    //println!("{:?}", elapsed);

    logging::log("integer_padded_paral total", elapsed);

    for i in 0..s.len() {
        s_clear[i] = client_key.decrypt(&s[i]);
    }
    for i in 0..b.len() {
        b_clear[i] = client_key.decrypt(&b[i]);
    }
}

pub fn volume_match(
    s : &mut Vec<Ciphertext>,
    b : &mut Vec<Ciphertext>,
    s16 : &mut Vec<Ciphertext>,
    b16 : &mut Vec<Ciphertext>,
    NUM_BLOCK: usize,
    server_key: &ServerKey
    //client_key: &RadixClientKey
) {
    // Init variables
    let mut sell_vol  = server_key.create_trivial_zero_radix(NUM_BLOCK*2);
    let mut buy_vol  = server_key.create_trivial_zero_radix(NUM_BLOCK*2);
    
    // Sum into S and B in paralell
    //let now = Instant::now();
    
    join(
        || (for i in 0..s.len() {add(&mut sell_vol, &mut s[i], server_key);}), 
        || (for i in 0..b.len() {add(&mut buy_vol,&mut b[i], server_key);})
    );

    //let elapsed = now.elapsed();
    //logging::log("integer_padded_paral summing", elapsed);
    //println!("integer_padded_paral : Summing: {elapsed:?}");
    
    // Min of S and B
    //let now = Instant::now();

    sell_vol = server_key.smart_min_parallelized(&mut sell_vol, &mut buy_vol);
    buy_vol = sell_vol.clone();
    
    //let elapsed = now.elapsed();
    //logging::log("integer_padded_paral leftvols", elapsed);
    //println!("integer_padded_paral : Setting up leftvols: {elapsed:.2?}");
    
    // Calculate new s and b <- Parallalise this
    //let mut min_dur = Duration::new(0,0);
    //let mut sub_dur = Duration::new(0,0);
    //let now = Instant::now();
    
    join(
        ||(for i in 0..s.len()
        {
            //let now2 = Instant::now();

            //s[i] = min(&mut sell_vol, &mut s[i], server_key);
            s16[i] = server_key.smart_min_parallelized(&mut s16[i], &mut sell_vol);
            s[i] = from2NtoNbit(&mut s16[i]);

            //min_dur += now2.elapsed();
            //let now2 = Instant::now();

            sub(&mut sell_vol, &mut s[i], server_key);
            //sub(&mut sell_vol, &mut s16[i], server_key);

            //sub_dur += now2.elapsed();
        }),
        ||(for i in 0..b.len()
        {
            //b[i] = min(&mut buy_vol, &mut b[i], server_key);
            b[i] = from2NtoNbit(&mut server_key.smart_min_parallelized(&mut buy_vol, &mut b16[i]));

            sub(&mut buy_vol,&mut b[i], server_key);
        })
    );
    
    //let elapsed = now.elapsed();
    
    //println!("integer_padded_paral : Subtracting only s: {sub_dur:?}");
    //println!("integer_padded_paral : Min only s: {min_dur:?}");
    //println!("integer_padded_paral : Subtracting and min: {elapsed:.2?}");
    //logging::log("integer_padded_paral loop", elapsed);

}

fn fromNto2Nbit(x: &Ciphertext, server_key: &ServerKey) -> Ciphertext
{
    let NUM_BLOCK: usize = x.blocks().len();
    let temp: Ciphertext = server_key.create_trivial_zero_radix(NUM_BLOCK);

    let mut u = Vec::new();
    u.extend_from_slice(x.blocks());
    u.extend_from_slice(temp.blocks());

    IntegerCiphertext::from_blocks(u)
}

fn from2NtoNbit(x: &mut Ciphertext) -> Ciphertext
{
    IntegerCiphertext::from_blocks(x.blocks()[0..x.blocks().len()/2].to_vec())
}

fn add(
    a: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey
) {
    /*let mut v: Vec<usize> = Vec::new();
    for block in b.blocks_mut().iter_mut() {
        v.push(block.degree.0);
    }
    println!("{:?}", v);
    v.clear();
    for block in a.blocks_mut().iter_mut() {
        v.push(block.degree.0);
    }
    println!("{:?}", v);*/
    if !server_key.is_add_possible(a, &b) {
        server_key.full_propagate_parallelized(a);
    }
    server_key.unchecked_add_assign(a, &b);
}

fn sub(
    a: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey,
    //client_key: &RadixClientKey
) {
    /*let mut v: Vec<u64> = Vec::new();
    v.push(client_key.decrypt(a));
    v.push(client_key.decrypt(b));*/

    if !server_key.is_sub_possible(a, b) || !server_key.is_neg_possible(b) {
        server_key.full_propagate_parallelized(b);
    }
    if !server_key.is_sub_possible(a, b) {
        server_key.full_propagate_parallelized(a);
    }

    server_key.unchecked_sub_assign(a, b);
    //server_key.smart_scalar_add_assign_parallelized(a, 255*256);
    server_key.smart_scalar_sub_assign_parallelized(a, 256);

    /*v.push(client_key.decrypt(a));
    println!("a: {}, b: {}, a-b: {}", v[0], v[1], v[2]);*/
}

fn min(
    a: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey,
) -> Ciphertext {
    let mut b16 = fromNto2Nbit(b, server_key);

    let mut min16 = server_key.smart_min_parallelized(a, &mut b16);
    from2NtoNbit(&mut min16)
}
