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

fn random_concat(n: usize, len: usize, alpha: &[u8]) -> Vec<u8> {
    let strings = random_strings(n, len, alpha);
    let mut concat = strings.join(&b'$');
    concat.push(b'$');
    concat
}

const N: usize = 10000;
const LEN: usize = 20;

#[divan::bench]
fn merge_test(bencher: Bencher) {
    let str0 = random_concat(N/2, LEN, ALPHABET);
    let str1 = random_concat(N/2, LEN, ALPHABET);

    let bwt0 = bwt::run_bwt(&str0);
    let bwt1 = bwt::run_bwt(&str1);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(&bwt0), black_box(&bwt1));
    })
}

// worst case performance, where strings are duplicated
#[divan::bench]
fn repetitive_merge_test(bencher: Bencher) {
    let str = random_concat(N/2, LEN, ALPHABET);

    let bwt = bwt::run_bwt(&str);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(&bwt), black_box(&bwt));
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

    let bwt0 = bwt::run_bwt(&str0);
    let bwt1 = bwt::run_bwt(&str1);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(&bwt0), black_box(&bwt1));
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
