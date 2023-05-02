#![allow(non_snake_case)]
use tfhe::integer::{ServerKey, gen_keys_radix,  ciphertext::BaseRadixCiphertext};
use tfhe::shortint::{CiphertextBase, ciphertext::KeyswitchBootstrap};
use rayon::{join};
use tfhe::integer::ciphertext::IntegerCiphertext;
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
use std::time::{Instant, Duration};

type Ciphertext = BaseRadixCiphertext<CiphertextBase<KeyswitchBootstrap>>;

pub fn run(s_clear: &mut Vec<u64>, b_clear: &mut Vec<u64>, _NUM_BLOCK: usize) {
    let (client_key, server_key) = gen_keys_radix(&PARAM_MESSAGE_2_CARRY_2, _NUM_BLOCK);
    let mut s = Vec::with_capacity(s_clear.len());
    let mut b = Vec::with_capacity(b_clear.len());

    let mut s16 = Vec::with_capacity(s_clear.len());
    let mut b16 = Vec::with_capacity(b_clear.len());

    for i in 0..s_clear.len() {
        s.push(client_key.encrypt(s_clear[i]));
        s16.push(fromNto2Nbit(&mut s[i], &server_key));
    }

    for i in 0..s_clear.len() {
        b.push(client_key.encrypt(b_clear[i]));
        b16.push(fromNto2Nbit(&mut b[i], &server_key));
    }

    let now = Instant::now();
    println!("----------------------\nRunning integer_padded_paral");

    volume_match(&mut s, &mut b, &mut s16, &mut b16, _NUM_BLOCK , &server_key);

    let elapsed = now.elapsed();
    println!("Time for the intgere_padded_paral: {elapsed:.2?}\n----------------------");

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
    server_key: &ServerKey,
    //client_key : &RadixClientKey
) {
    // Init variables
    //let NUM_BLOCK: usize = s[0].blocks().len(); //we know this

    let mut sell_vol  = server_key.create_trivial_zero_radix(NUM_BLOCK*2);
    let mut buy_vol  = server_key.create_trivial_zero_radix(NUM_BLOCK*2);
    
    // Sum into S and B in paralell
    let now = Instant::now();

    join(
        || (for i in 0..s.len() {add(&mut sell_vol, &mut s[i], server_key);}), 
        || (for i in 0..b.len() {add(&mut buy_vol,&mut b[i], server_key);})
    );

    let elapsed = now.elapsed();
    println!("integer_padded_paral : Summing s and b: {elapsed:.2?}");
    
    // Min of S and B
    let now = Instant::now();

    sell_vol = server_key.smart_min_parallelized(&mut sell_vol, &mut buy_vol);
    buy_vol = sell_vol.clone();
    
    let elapsed = now.elapsed();
    println!("integer_padded_paral : Setting up leftvols: {elapsed:.2?}");
    
    // Calculate new s and b <- Parallalise this
    let mut min_dur = Duration::new(0,0);
    let mut sub_dur = Duration::new(0,0);
    
    join(
        ||(for i in 0..s.len()
        {
            let now2 = Instant::now();

            //s[i] = min(&mut sell_vol, &mut s[i], server_key);
            s[i] = from2NtoNbit(&mut server_key.smart_min_parallelized(&mut sell_vol, &mut s16[i]));

            min_dur += now2.elapsed();
            let now2 = Instant::now();

            sub(&mut sell_vol, &mut s[i], server_key);

            sub_dur += now2.elapsed();
        }),
        ||(for i in 0..b.len()
        {
            //b[i] = min(&mut buy_vol, &mut b[i], server_key);
            b[i] = from2NtoNbit(&mut server_key.smart_min_parallelized(&mut buy_vol, &mut b16[i]));

            sub(&mut buy_vol,&mut b[i], server_key);
        })
    );
    
    let elapsed = now.elapsed();
    
    println!("integer_padded_paral : Subtracting only s: {sub_dur:.2?}");
    println!("integer_padded_paral : Min only s: {min_dur:.2?}");
    println!("integer_padded_paral : Subtracting and min: {elapsed:.2?}");

}

fn fromNto2Nbit(x: &mut Ciphertext, server_key: &ServerKey) -> Ciphertext
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
    b: &Ciphertext,
    server_key: &ServerKey
) {     
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
        //let mut b16 = from8to16bit(b, server_key);
        server_key.smart_sub_assign_parallelized(a, b);
        server_key.smart_scalar_add_assign(a, 255*256);
        /*v.push(client_key.decrypt(a));

        println!("a: {} b: {} a-b: {}", v[0], v[1], v[2]);*/
    }

fn min(
    a: &mut Ciphertext,
    b: &mut Ciphertext,
    server_key: &ServerKey,
    //client_key: &RadixClientKey
) -> Ciphertext {
        let mut b16 = fromNto2Nbit(b, server_key);

        let mut min16 = server_key.smart_min_parallelized(a, &mut b16);
        let min8: Ciphertext = from2NtoNbit(&mut min16);

        /*let mut v: Vec<u64> = Vec::new();
        
        v.push(client_key.decrypt(a));
        v.push(client_key.decrypt(b));
        v.push(client_key.decrypt(&mut b16));
        v.push(client_key.decrypt(&min8));
        println!("a: {} b: {} b16: {} min: {}", v[0], v[1], v[2], v[3]);*/

        min8
    }