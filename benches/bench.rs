use divan::{black_box, Bencher};
use rand::{rngs::StdRng, Rng, SeedableRng};
use bwt_merge::bwt;

fn main() {
    divan::main();
}

const ALPHABET: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

// Creates n random strings of length len
// Separates strings by $, and concatenates them
fn random_strings(n: usize, len: usize) -> Vec<Vec<u8>> {
    let mut rng = StdRng::seed_from_u64(123);
    let mut strings: Vec<Vec<u8>> = Vec::new();
    for _ in 0..n {
        let mut string = Vec::new();
        for _ in 0..len {
            string.push(ALPHABET[rng.gen_range(0..62)]);
        }
        strings.push(string);
    }
    strings
}

fn random_concat(n: usize, len: usize) -> Vec<u8> {
    let strings = random_strings(n, len);
    let mut concat = strings.join(&b'$');
    concat.push(b'$');
    concat
}

const N: usize = 10000;
const LEN: usize = 20;

#[divan::bench]
fn merge_test(bencher: Bencher) {
    let str0 = random_concat(N/2, LEN);
    let str1 = random_concat(N/2, LEN);

    let bwt0 = bwt::run_bwt(str0);
    let bwt1 = bwt::run_bwt(str1);
    bencher.bench_local(move || {
        bwt::bwt_merge(black_box(bwt0.clone()), black_box(bwt1.clone()));
    })
}

#[divan::bench]
fn rebuild_test(bencher: Bencher) {
    let str = random_concat(N, LEN);

    bencher.bench_local(move || {
        bwt::run_bwt(black_box(str.clone()));
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
