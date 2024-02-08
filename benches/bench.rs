use divan::{black_box, Bencher};
use rand::{rngs::StdRng, Rng, SeedableRng};
use bwt_merge::bwt;

fn main() {
    divan::main();
}

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const ALPHABET_SMALL: &[u8] = b"ACGT";

// Creates n random strings of length len
// Separates strings by $, and concatenates them
fn random_strings(n: usize, len: usize, alpha: &[u8]) -> Vec<Vec<u8>> {
    let mut rng = StdRng::seed_from_u64(123);
    let mut strings: Vec<Vec<u8>> = Vec::new();
    for _ in 0..n {
        let mut string = Vec::new();
        for _ in 0..len {
            string.push(ALPHABET[rng.gen_range(0..alpha.len())]);
        }
        strings.push(string);
    }
    strings
}

fn merge_strs(strs: Vec<Vec<u8>>) -> Vec<u8> {
    let mut concat = strs.join(&b'\n');
    concat.push(b'\n');
    concat
}

fn random_concat(n: usize, len: usize, alpha: &[u8]) -> Vec<u8> {
    let strings = random_strings(n, len, alpha);
    merge_strs(strings)
}

const N: usize = 10000;
const LEN: usize = 20;

#[divan::bench]
fn merge_test(bencher: Bencher) {
    let str0 = random_concat(N/2, LEN, ALPHABET);
    let str1 = random_concat(N/2, LEN, ALPHABET);

    let data0 = bwt::run_bwt(&str0);
    let data1 = bwt::run_bwt(&str1);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(&data0), black_box(&data1));
    })
}

// worst case performance, where strings are duplicated
#[divan::bench]
fn repetitive_merge_test(bencher: Bencher) {
    let strs = random_strings(N/2, LEN, ALPHABET);
    let mut rng = StdRng::seed_from_u64(123);

    // modify last character
    let str_mod = strs.clone().into_iter().map(|x| {
        let mut x = x.clone();
        let rand_char = ALPHABET[rng.gen_range(0..ALPHABET.len())];
        let x_ind = x.len() - 1;
        x[x_ind] = rand_char;
        x
    }).collect();

    let str0 = merge_strs(strs);
    let str1 = merge_strs(str_mod);

    let data0 = bwt::run_bwt(&str0);
    let data1 = bwt::run_bwt(&str1);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(&data0), black_box(&data1));
    })
}

#[divan::bench]
fn rebuild_test(bencher: Bencher) {
    let str = random_concat(N, LEN, ALPHABET);

    bencher.bench_local(move || {
        bwt::run_bwt(black_box(&str));
    })
}

// test on smaller alphabet
#[divan::bench]
fn merge_test_small(bencher: Bencher) {
    let str0 = random_concat(N/2, LEN, ALPHABET_SMALL);
    let str1 = random_concat(N/2, LEN, ALPHABET_SMALL);

    let data0 = bwt::run_bwt(&str0);
    let data1 = bwt::run_bwt(&str1);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(&data0), black_box(&data1));
    })
}

#[divan::bench]
fn rebuild_test_small(bencher: Bencher) {
    let str = random_concat(N, LEN, ALPHABET_SMALL);

    bencher.bench_local(move || {
        bwt::run_bwt(black_box(&str));
    })
}

/*
#[divan::bench]
fn scratch_test(bencher: Bencher) {
    let strings = random_strings(N, LEN);
    bencher.bench_local(move || {
        bwt::scratch_build_bwt(&strings);
    })
}
*/
