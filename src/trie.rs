use std::cmp::{max, min};

use serde::{Deserialize, Serialize};

// extra bits to store in trie
// in general if you want to merge n tries, you need log2(n) extra bits
const EXTRA_BITS: usize = 8;

#[derive(Serialize, Deserialize)]
pub struct BinaryTrieNode {
    pub left: Option<Box<BinaryTrieNode>>,
    pub right: Option<Box<BinaryTrieNode>>,
    pub data: Vec<usize>,
}

impl BinaryTrieNode {
    pub fn new() -> BinaryTrieNode {
        BinaryTrieNode {
            left: None,
            right: None,
            data: Vec::new(),
        }
    }
}

impl Default for BinaryTrieNode {
    fn default() -> Self {
        Self::new()
    }
}

// note: it's difficult to support insert on this trie,
// since we stop storing anything past the LCP
// so we can't differentiate between two strings that share a prefix after build

pub fn build_binary_trie(strs: &[Vec<u8>], inds: &[Vec<usize>]) -> BinaryTrieNode {
    // big endian
    let get_bit = |stri: usize, i: usize| -> bool {
        let chr = i / 8;
        let bit = 7 - (i % 8);
        (strs[stri][chr] >> bit) & 1 == 1
    };

    // calculate LCPs
    let mut lcp = vec![0; strs.len() + 1];
    // lcp[i] contains lcp of strs[i-1] and strs[i]
    // lcp[0] := lcp[n] := 0
    for i in 0..strs.len() - 1 {
        let mut j = 0;
        while j < strs[i].len() * 8
            && j < strs[i + 1].len() * 8
            && get_bit(i, j) == get_bit(i + 1, j)
        {
            j += 1;
        }
        lcp[i + 1] = j;
    }

    // build trie
    let mut root = BinaryTrieNode::new();
    for i in 0..strs.len() {
        let node_depth = min(max(lcp[i], lcp[i + 1]) + 1 + EXTRA_BITS, strs[i].len() * 8);
        let mut node = &mut root;
        for j in 0..node_depth {
            if !get_bit(i, j) {
                if node.left.is_none() {
                    node.left = Some(Box::new(BinaryTrieNode::new()));
                }
                node = node.left.as_mut().unwrap();
            } else {
                if node.right.is_none() {
                    node.right = Some(Box::new(BinaryTrieNode::new()));
                }
                node = node.right.as_mut().unwrap();
            }
        }
        node.data.extend(&inds[i]);
    }

    root
}

pub fn merge_tries(t1: &BinaryTrieNode, t2: &BinaryTrieNode) -> BinaryTrieNode {
    let mut output = BinaryTrieNode::new();
    todo!();
}

// Query the trie for matching indices
// Note that if string does not exist it may return results that don't match,
// but at most one, so you can check manually
pub fn query_string<'a>(root: &'a BinaryTrieNode, query: &[u8]) -> Vec<usize> {
    let get_bit = |i: usize| -> bool {
        let chr = i / 8;
        let bit = 7 - (i % 8);
        (query[chr] >> bit) & 1 == 1
    };

    let mut node = root;
    for i in 0..query.len() * 8 {
        if !get_bit(i) {
            if node.left.is_none() {
                return node.data.clone();
            }
            node = node.left.as_ref().unwrap();
        } else {
            if node.right.is_none() {
                return node.data.clone();
            }
            node = node.right.as_ref().unwrap();
        }
    }

    node.data.clone()
}
