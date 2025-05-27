use std::sync::{atomic::{AtomicUsize, Ordering}, mpsc, Arc};
use std::thread;

use crate::math::big_int::{BigInt, BigIntMod};
use super::{KEY_SIZE, MILLER_ROUND};

fn check_candidate_prime(num: BigInt<KEY_SIZE>, primes: &Vec<u64>) -> bool {
    for prime in primes {
        if num.mod_u64(*prime) == 0 {
            return false;
        }
    };
    return true;
}

fn generate_small_primes(limit: usize) -> Vec<u64> {
    let mut sieve = vec![true; limit + 1];
    let mut primes = Vec::new();

    for i in 2..=limit {
        if sieve[i] {
            primes.push(i as u64);
            for multiple in (i * i..=limit).step_by(i) {
                sieve[multiple] = false;
            }
        }
    }
    primes
}

fn check_prime(num: BigInt<KEY_SIZE>, round: usize, found_total: &AtomicUsize, n: usize) -> bool {
    let mut d = num - BigInt::<KEY_SIZE>::from_num(1);
    let mut r = 0;
    while !d.is_odd() {
        d = d >> 1;
        r += 1;
    }
    let mu = BigIntMod::<KEY_SIZE>::calculate_mu(num.clone());
    for _ in 0..round {
        if found_total.load(Ordering::Relaxed) >= n {
            return false;
        }
        let a = BigIntMod::new_reduce(BigInt::<KEY_SIZE>::rand(1, KEY_SIZE / 2 - 2), num.clone(), mu.clone());
        let mut x = a.pow(d.clone());
        if !(x.integer == BigInt::<KEY_SIZE>::from_num(1) || x.integer == num - BigInt::<KEY_SIZE>::from_num(1)) {
            return false;
        }
        for _ in 0..(r - 1) {
            if found_total.load(Ordering::Relaxed) >= n {
                return false;
            }
            x = x * x;
            if x.integer != num - BigInt::<KEY_SIZE>::from_num(1) {
                return false;
            }
        }
    }

    true
}

pub fn thread_generate_prime(found_total: &AtomicUsize, n: usize) -> Option<BigInt<KEY_SIZE>> {
    let mut small_primes = generate_small_primes(65536);
    let mut prime = BigInt::<KEY_SIZE>::rand(16, KEY_SIZE / 2 - 2);
    let mod6 = prime.mod_u64(6);
    prime = prime + BigInt::<KEY_SIZE>::from_num(7 - mod6);

    let mut toggle = true;
    loop {
        if found_total.load(Ordering::Relaxed) >= n {
            return None;
        }

        if toggle {
            prime = prime + BigInt::<KEY_SIZE>::from_num(4);
        } else {
            prime = prime + BigInt::<KEY_SIZE>::from_num(2);
        }
        toggle = !toggle;

        if !check_candidate_prime(prime, &mut small_primes) { continue; }
        if check_prime(prime, MILLER_ROUND, found_total, n) { break; }
    }
    Some(prime)
}

pub fn generate_primes(n: usize) -> Vec<BigInt<KEY_SIZE>> {
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let (tx, rx) = mpsc::channel::<BigInt<KEY_SIZE>>();
    let found_total = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for _ in 0..cores {
        let tx = tx.clone();
        let found_total = Arc::clone(&found_total);

        handles.push(thread::spawn(move || {
            loop {
                if found_total.load(Ordering::Relaxed) >= n {
                    break;
                }

                if let Some(prime) = thread_generate_prime(&found_total, n) {
                    println!("Found prime");
                    if found_total.fetch_add(1, Ordering::Relaxed) < n {
                        let _ = tx.send(prime);
                    } else {
                        break;
                    }
                }
            }
        }));
    }

    drop(tx); // Close the channel to allow the receiver to exit when all threads are done    

    let mut primes = Vec::with_capacity(n);
    for _ in 0..n {
        if let Ok(prime) = rx.recv() {
            primes.push(prime);
        }
    }

    for handle in handles {
        let _ = handle.join();
    }

    primes
}