use std::collections::VecDeque;

use bit_vec::BitVec;
use libdivsufsort_rs::divsufsort64;

type BWT = Vec<u8>;

// Compute the BWT of a string, using the divsufsort crate.
pub fn run_bwt(input: &Vec<u8>) -> BWT {
    let sa = divsufsort64(&input).unwrap();

    let mut bwt = Vec::new();
    for i in 0..sa.len() {
        if sa[i] == 0 {
            bwt.push(input[input.len() - 1]);
        } else {
            bwt.push(input[(sa[i] - 1) as usize]);
        }
    }

    return bwt;
}

// Merge two BWTs using our algorithm.
pub fn bwt_merge(bwt0: &BWT, bwt1: &BWT) -> BWT {
    // construct character counts array
    let mut counts: [usize; 256] = [0; 256];
    let mut interleave = BitVec::from_elem(bwt0.len() + bwt1.len(), true);
    for i in 0..bwt0.len() {
        counts[bwt0[i] as usize] += 1;
        interleave.set(i, false);
    }
    for i in 0..bwt1.len() {
        counts[bwt1[i] as usize] += 1;
    }

    // construct character starts array
    let mut starts: [usize; 256] = [0; 256];
    let mut sum = 0;
    for i in 0..256 {
        starts[i] = sum;
        sum += counts[i];
    }

    loop {
        let mut ind: [usize; 2] = [0, 0];

        let mut offsets = starts.clone();
        let mut new_interleave = BitVec::from_elem(interleave.len(), false);
        for i in 0..interleave.len() {
            if interleave[i] {
                new_interleave.set(offsets[bwt1[ind[1]] as usize], true);
                offsets[bwt1[ind[1]] as usize] += 1;
                ind[1] += 1;
            } else {
                offsets[bwt0[ind[0]] as usize] += 1;
                ind[0] += 1;
            }
        }

        if new_interleave == interleave {
            break;
        }
        interleave = new_interleave;
    }

    // construct bwt
    let mut bwt = Vec::new();
    let mut ind0 = 0;
    let mut ind1 = 0;
    for i in 0..interleave.len() {
        if interleave[i] {
            bwt.push(bwt1[ind1]);
            ind1 += 1;
        } else {
            bwt.push(bwt0[ind0]);
            ind0 += 1;
        }
    }
    return bwt;
}

// Build the BWT of N strings by merging
pub fn scratch_build_bwt(input: &Vec<Vec<u8>>) -> BWT {
    // build BWT of each individual string
    let mut bwt_queue: VecDeque<Vec<u8>> = VecDeque::new();
    for i in 0..input.len() {
        let mut str = input[i].clone();
        str.push(b'$');
        bwt_queue.push_back(run_bwt(&str));
    }

    // merge BWTs
    while bwt_queue.len() > 1 {
        let bwt0 = bwt_queue.pop_front().unwrap();
        let bwt1 = bwt_queue.pop_front().unwrap();
        bwt_queue.push_back(bwt_merge(&bwt0, &bwt1));
    }

    return bwt_queue.pop_front().unwrap();
}
