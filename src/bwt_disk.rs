use std::{fs::File, io::{BufRead, Write}};

use bit_vec::BitVec;
use rand::seq::SliceRandom;

use crate::bwt::run_bwt;

// generate subsets of input file of certain sizes using naive algorithm
// and calculate bwt and write to file
const SIZES: &[usize] = &[1024, 4096, 16384, 65536, 262144, 1048576, 2097152, usize::MAX];
pub fn generate_test_files(input_file: &str, output_path: &str) {
    let line_count = std::fs::read_to_string(input_file).unwrap().lines().count();
    let mut rng = rand::thread_rng();

    for xsize in SIZES {
        // time
        let start = std::time::Instant::now();

        let mut size = *xsize;
        if size == usize::MAX {
            size = line_count / 2;
        }
        
        // generate random sample indices
        let mut sample_indices: Vec<usize> = (0..line_count).collect();
        sample_indices.shuffle(&mut rng);
        sample_indices.truncate(size * 2);

        // create two output files
        for ind in 0..2 {
            let mut indices = sample_indices[ind*size..(ind+1)*size].to_vec();
            indices.sort();

            let mut strs: Vec<u8> = Vec::new();
            let mut indices_i = 0;
            for (i, line) in std::fs::read_to_string(input_file).unwrap().lines().enumerate() {
                if indices_i < indices.len() && i == indices[indices_i] {
                    strs.extend_from_slice(line.as_bytes());
                    strs.push(b'\n');
                    indices_i += 1;
                }
            }
            
            let bwt = run_bwt(&strs);

            let output_file = format!("{}/{}_{}.bwt", output_path, size, ind);
            let output_index_file = format!("{}/{}_{}.index", output_path, size, ind);
            let output_counts_file = format!("{}/{}_{}.counts", output_path, size, ind);

            std::fs::write(output_file, bwt.0).unwrap();
            let index_str = bwt.1.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n");
            std::fs::write(output_index_file, index_str).unwrap();
            let counts_str = bwt.2.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n");
            std::fs::write(output_counts_file, counts_str).unwrap();
        }

        // generate full sampled file
        sample_indices.sort();

        let mut strs: Vec<u8> = Vec::new();
        let mut indices_i = 0;
        for (i, line) in std::fs::read_to_string(input_file).unwrap().lines().enumerate() {
            if indices_i < sample_indices.len() && i == sample_indices[indices_i] {
                strs.extend_from_slice(line.as_bytes());
                strs.push(b'\n');
                indices_i += 1;
            }
        }

        let output_file = format!("{}/{}_full.txt", output_path, size);
        std::fs::write(output_file, strs).unwrap();
        println!("generated test files for size {}", size);

        let duration = start.elapsed();
        println!("time to generate for size {}: {:?}", size, duration);
    }
}

// Size of buffer for reading files
const BUFFER_SIZE: usize = 1024*1024;

// Compute the interleave of two BWTs, stored on disk.
// BWT paths should include the .bwt extension
fn compute_interleave_disk(bwt0_path: &str, bwt1_path: &str, counts: &[usize; 256]) -> BitVec {
    // get lengths of bwt files
    let bwt0_len = File::open(bwt0_path).unwrap().metadata().unwrap().len() as usize;
    let bwt1_len = File::open(bwt1_path).unwrap().metadata().unwrap().len() as usize;

    // construct character starts array
    let mut starts: [usize; 256] = [0; 256];
    let mut sum = 0;
    for i in 0..256 {
        starts[i] = sum;
        sum += counts[i];
    }

    let mut interleave = BitVec::from_elem(bwt0_len + bwt1_len, true);
    for i in 0..bwt0_len {
        interleave.set(i, false);
    }

    loop {
        let mut ind: [usize; 2] = [0, 0];

        // reset readers
        let mut bwt0_reader = std::io::BufReader::with_capacity(BUFFER_SIZE, File::open(bwt0_path).unwrap());
        let mut bwt1_reader = std::io::BufReader::with_capacity(BUFFER_SIZE, File::open(bwt1_path).unwrap());
        let mut bwt0 = bwt0_reader.fill_buf().unwrap();
        let mut bwt1 = bwt1_reader.fill_buf().unwrap();

        let mut offsets = starts.clone();
        let mut new_interleave = BitVec::from_elem(interleave.len(), false);
        for i in 0..interleave.len() {
            if interleave[i] {
                new_interleave.set(offsets[bwt1[ind[1]] as usize], true);
                offsets[bwt1[ind[1]] as usize] += 1;
                ind[1] += 1;

                if ind[1] == BUFFER_SIZE {
                    bwt1_reader.consume(BUFFER_SIZE);
                    bwt1 = bwt1_reader.fill_buf().unwrap();
                    ind[1] = 0;
                }
            } else {
                offsets[bwt0[ind[0]] as usize] += 1;
                ind[0] += 1;

                if ind[0] == BUFFER_SIZE {
                    bwt0_reader.consume(BUFFER_SIZE);
                    bwt0 = bwt0_reader.fill_buf().unwrap();
                    ind[0] = 0;
                }
            }
        }

        if new_interleave == interleave {
            break;
        }
        interleave = new_interleave;
    }
    interleave
}

// Merge two BWTs using our algorithm.
// Paths should be the paths to the extensionless files
pub fn bwt_merge_disk(bwt0_path: &str, bwt1_path: &str, output_path: &str) {
    // construct character counts array
    let mut counts: [usize; 256] = [0; 256];
    for (i, line) in std::fs::read_to_string(format!("{}.counts", bwt0_path)).unwrap().lines().enumerate() {
        counts[i] = line.parse().unwrap();
    }
    let num_newlines = counts[b'\n' as usize];
    for (i, line) in std::fs::read_to_string(format!("{}.counts", bwt1_path)).unwrap().lines().enumerate() {
        counts[i] += line.parse::<usize>().unwrap();
    }

    // get bwt file paths
    let bwt0_file_path = format!("{}.bwt", bwt0_path);
    let bwt1_file_path = format!("{}.bwt", bwt1_path);
    let interleave = compute_interleave_disk(bwt0_file_path.as_str(), bwt1_file_path.as_str(), &counts);

    // construct bwt

    // create bufreaders and bufwriters
    let mut bwt0_reader = std::io::BufReader::with_capacity(BUFFER_SIZE, File::open(bwt0_file_path).unwrap());
    let mut bwt1_reader = std::io::BufReader::with_capacity(BUFFER_SIZE, File::open(bwt1_file_path).unwrap());
    let mut bwt0 = bwt0_reader.fill_buf().unwrap();
    let mut bwt1 = bwt1_reader.fill_buf().unwrap();

    let line_ind0 = std::fs::read_to_string(format!("{}.index", bwt0_path)).unwrap();
    let line_ind1 = std::fs::read_to_string(format!("{}.index", bwt1_path)).unwrap();
    let mut line_ind0_iter = line_ind0.lines();
    let mut line_ind1_iter = line_ind1.lines();

    let mut bwt_writer = std::io::BufWriter::with_capacity(BUFFER_SIZE, File::create(format!("{}.bwt", output_path)).unwrap());
    let mut line_index_writer = std::io::BufWriter::with_capacity(BUFFER_SIZE, File::create(format!("{}.index", output_path)).unwrap());

    let mut ind0 = 0;
    let mut ind1 = 0;
    
    for i in 0..interleave.len() {
        if interleave[i] {
            bwt_writer.write(&[bwt1[ind1]]).unwrap();
            let line_ind = line_ind1_iter.next().unwrap().parse::<usize>().unwrap() + num_newlines;
            line_index_writer.write(format!("{}\n", line_ind).as_bytes()).unwrap();
            ind1 += 1;

            if ind1 == BUFFER_SIZE {
                bwt1_reader.consume(BUFFER_SIZE);
                bwt1 = bwt1_reader.fill_buf().unwrap();
                ind1 = 0;
            }
        } else {
            bwt_writer.write(&[bwt0[ind0]]).unwrap();
            let line_ind: usize = line_ind0_iter.next().unwrap().parse().unwrap();
            line_index_writer.write(format!("{}\n", line_ind).as_bytes()).unwrap();
            ind0 += 1;

            if ind0 == BUFFER_SIZE {
                bwt0_reader.consume(BUFFER_SIZE);
                bwt0 = bwt0_reader.fill_buf().unwrap();
                ind0 = 0;
            }
        }
    }

    // write counts
    std::fs::write(format!("{}.counts", output_path), counts.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n")).unwrap();
}

pub fn test_merge_disk(input_path: &str, output_path: &str, test_rebuild: bool) {
    let mut test_sizes = SIZES[0..SIZES.len()-1].to_vec();
    test_sizes.push(3719388); // full size

    for size in test_sizes.iter() {
        let bwt0_path = format!("{}/{}_0", input_path, size);
        let bwt1_path = format!("{}/{}_1", input_path, size);
        let output_path_n = format!("{}/{}_merged", output_path, size);

        // time merge
        let merge_start = std::time::Instant::now();
        bwt_merge_disk(&bwt0_path, &bwt1_path, &output_path_n);
        let merge_duration = merge_start.elapsed();
        println!("merge time for size {}: {:?}", size, merge_duration);

        if test_rebuild {
            // time full rebuild, including i/o times
            let rebuild_start = std::time::Instant::now();
            let full_text = std::fs::read(format!("{}/{}_full.txt", input_path, size)).unwrap().to_vec();
            let full_bwt = run_bwt(&full_text);
    
            let output_path = format!("{}/{}_merged_naive", output_path, size);
            std::fs::write(format!("{}.bwt", output_path), full_bwt.0).unwrap();
            std::fs::write(format!("{}.index", output_path), full_bwt.1.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n")).unwrap();
            std::fs::write(format!("{}.counts", output_path), full_bwt.2.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n")).unwrap();
            let rebuild_duration = rebuild_start.elapsed();
            println!("rebuild time for size {}: {:?}", size, rebuild_duration);
        }
    }
}
