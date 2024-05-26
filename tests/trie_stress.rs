// command to run: cargo test --test trie_stress --release -- --nocapture --test-threads=1 --ignored

use std::{io::Write, time::Instant};

use bwt_merge::trie::{self, hex_to_u8};
use rand::{prelude::SliceRandom, rngs::ThreadRng, Rng};
use serde::Serialize;

const INPUT_FILE: &str = "data/compacted_type_5";
const QUERY_COUNT: usize = 1000;
const ALPHABET: &[u8; 16] = b"0123456789abcdef";

fn read_input_file(incl_newline: bool, compress_hex: bool) -> Vec<Vec<u8>> {
    let input_raw = std::fs::read(INPUT_FILE).unwrap();
    let mut input;
    if compress_hex {
        input = input_raw
            .split(|&x| x == b'\n')
            .map(|x| hex_to_u8(std::str::from_utf8(x).unwrap()).unwrap())
            .collect::<Vec<_>>();
    } else {
        input = input_raw
            .split(|&x| x == b'\n')
            .map(|x| x.to_vec())
            .collect::<Vec<_>>();
    }

    // append \n
    if incl_newline {
        for line in input.iter_mut() {
            line.push(b'\n');
        }
    }
    input
}

fn split_input(rng: &mut ThreadRng, incl_newline: bool, compress_hex: bool) -> [Vec<Vec<u8>>; 2] {
    let input = read_input_file(incl_newline, compress_hex);
    let mut input1 = Vec::with_capacity(input.len());
    let mut input2 = Vec::with_capacity(input.len());
    for line in input.into_iter() {
        let which = rng.gen_bool(0.5);
        if which {
            input1.push(line.clone());
        } else {
            input2.push(line.clone());
        }
    }
    [input1, input2]
}

fn get_queries(
    rng: &mut ThreadRng,
    input_lines: &[Vec<u8>],
    block_size: usize,
) -> Vec<(Vec<u8>, i32)> {
    // choose QUERY_COUNT random indices
    let mut query_inds: Vec<usize> = Vec::with_capacity(QUERY_COUNT);
    for _ in 0..QUERY_COUNT {
        query_inds.push(rng.gen_range(0..input_lines.len()));
    }
    let mut query_strs = query_inds
        .iter()
        .map(|&i| (input_lines[i].clone(), (i / block_size) as i32))
        .collect::<Vec<_>>();

    // create random strings that don't exist (with high probability)
    let str_length = input_lines.get(0).unwrap().len();
    for _ in 0..QUERY_COUNT {
        let mut new_str = vec![0; str_length];
        for i in 0..str_length {
            new_str[i] = ALPHABET[rng.gen_range(0..ALPHABET.len())];
        }
        query_strs.push((new_str, -1));
    }

    // shuffle queries
    query_strs.shuffle(rng);
    query_strs
}

fn serialize_and_compress<T: Clone + Serialize>(trie: &trie::BinaryTrieNode<T>) {
    // check serialize size
    let serialized = bincode::serialize(&trie).unwrap();
    println!("trie size: {}", serialized.len());

    // check compressed size
    let mut encoder = zstd::stream::Encoder::new(Vec::new(), 0).unwrap();
    encoder.write_all(&serialized).unwrap();
    let compressed = encoder.finish().unwrap();
    println!("compressed size: {}", compressed.len());
}

#[ignore]
#[test]
fn stress_no_newline() {
    stress_test(false, 8, 1, false);
}

#[ignore]
#[test]
fn stress_hex() {
    stress_test(false, 8, 1, true);
}

#[ignore]
#[test]
fn stress_hex_no_extra_bits() {
    stress_test(false, 0, 1, true);
}

#[ignore]
#[test]
fn stress_newline() {
    stress_test(true, 8, 1, false);
}

#[ignore]
#[test]
fn stress_no_extra_bits() {
    stress_test(true, 0, 1, false);
}

#[ignore]
#[test]
fn stress_some_extra_bits() {
    stress_test(true, 4, 1, false);
}

#[ignore]
#[test]
fn stress_block_size() {
    stress_test(true, 8, 64, false);
}

fn stress_test(incl_newline: bool, extra_bits: usize, block_size: usize, compress_hex: bool) {
    let input_lines = read_input_file(incl_newline, compress_hex);

    // use block_size to compress inds
    let inds: Vec<Vec<usize>> = (0..input_lines.len())
        .map(|x| vec![x / block_size])
        .collect();

    let start = Instant::now();
    let trie = trie::BinaryTrieNode::build_extra(&input_lines, &inds, extra_bits);
    let duration = start.elapsed();
    println!("trie build time: {:?}", duration);

    serialize_and_compress(&trie);

    // test on random queries
    let mut rng = rand::thread_rng();
    let query_strs = get_queries(&mut rng, &input_lines, block_size);

    let mut fp_count_exists = 0;
    let mut fp_count_nonexists = 0;
    for (query, index) in query_strs.iter() {
        let res = trie.query(query);

        if *index != -1 {
            // println!("query {:?}, res: {:?}, index: {:?}", query, res, index);
            assert!(res.contains(&(*index as usize)));
            fp_count_exists += res.len() - 1;
        } else {
            fp_count_nonexists += res.len();
        }
    }

    println!(
        "false positives: {} for exists, {} for nonexists",
        fp_count_exists, fp_count_nonexists
    );
}

#[ignore]
#[test]
fn stress_merge_newline() {
    stress_merge(true, false, false);
}

#[ignore]
#[test]
fn stress_merge_no_newline() {
    stress_merge(false, false, false);
}

#[ignore]
#[test]
fn stress_merge_hex() {
    stress_merge(false, false, true);
}

#[ignore]
#[test]
fn stress_extend_newline() {
    stress_merge(true, true, false);
}

#[ignore]
#[test]
fn stress_extend_no_newline() {
    stress_merge(false, true, false);
}

fn stress_merge(incl_newline: bool, extend: bool, compress_hex: bool) {
    let mut rng = rand::thread_rng();
    let [input1, input2] = split_input(&mut rng, incl_newline, compress_hex);
    let inds1: Vec<Vec<(usize, bool)>> = (0..input1.len()).map(|x| vec![(x, false)]).collect();
    let inds2: Vec<Vec<(usize, bool)>> = (0..input2.len()).map(|x| vec![(x, true)]).collect();

    let mut trie1 = trie::BinaryTrieNode::build(&input1, &inds1);
    let trie2 = trie::BinaryTrieNode::build(&input2, &inds2);

    let start = Instant::now();
    let merged;
    if extend {
        trie1.extend(trie2);
        merged = trie1;
    } else {
        merged = trie::merge_tries(&trie1, &trie2);
    }
    let duration = start.elapsed();
    println!("merge time: {:?}", duration);

    // note: should be larger since we are storing an extra bool
    serialize_and_compress(&merged);

    // get queries and label with which input
    let query_strs = get_queries(&mut rng, &input1, 1)
        .into_iter()
        .map(|(query, index)| (query, (index, false)))
        .chain(
            get_queries(&mut rng, &input2, 1)
                .into_iter()
                .map(|(query, index)| (query, (index, true))),
        )
        .collect::<Vec<_>>();

    let mut fp_count_exists = 0;
    let mut fp_count_nonexists = 0;
    for (query, (index, which)) in query_strs.into_iter() {
        let res = merged.query(&query);

        if index != -1 {
            // println!("query {:?}, res: {:?}, index: {:?}", query, res, index);
            assert!(res.contains(&(index as usize, which)));
            fp_count_exists += res.len() - 1;
        } else {
            fp_count_nonexists += res.len();
        }
    }

    println!(
        "false positives: {} for exists, {} for nonexists",
        fp_count_exists, fp_count_nonexists
    );
}
