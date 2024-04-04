use bwt_merge::trie;

#[test]
fn build_and_query() {
    let mut strs = [
        "abcde", "abcdg", "abcd", "two", "acbde", "bbbbb", "bbbbbb", "abd", "abe", "two", "!@#$%",
        "",
    ];
    strs.sort();

    let strs_as_u8: Vec<Vec<u8>> = strs.iter().map(|s| s.as_bytes().to_vec()).collect();
    let inds: Vec<Vec<usize>> = (0..strs.len()).map(|x| vec![x]).collect();
    let trie = trie::BinaryTrieNode::build(&strs_as_u8, &inds);

    let queries = ["abcde", "abcd", "bbbbb", "abcdf", "abc", "two", "twa"];
    for query in queries.iter() {
        let query_u8 = query.as_bytes().to_vec();
        let res = trie.query(&query_u8);

        println!("query {}, res: {:?}", query, res);

        // make sure behavior is correct: allow false positives, but no false negatives
        let actual_inds = strs
            .iter()
            .enumerate()
            .filter(|(_, s)| s.eq(&query))
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        for i in actual_inds.iter() {
            println!("query {}, checking {:?} (index {})", query, strs[*i], *i);
            assert!(res.contains(i));
        }
    }
}

#[test]
fn merge_and_query() {
    let mut strs1 = [
        "abcde", "abcdg", "abcd", "two", "acbde", "bbbbb", "bbbbbb", "abd", "abe", "!@#$",
    ];
    strs1.sort();

    let mut strs2 = [
        "abcde", "abcdf", "ab", "two", "two", "bbbb", "bbbbbb", "abd", "abz", "",
    ];
    strs2.sort();

    let strs1_as_u8: Vec<Vec<u8>> = strs1.iter().map(|s| s.as_bytes().to_vec()).collect();
    let inds1: Vec<Vec<(usize, usize)>> = (0..strs1.len()).map(|x| vec![(1, x)]).collect();
    let trie1 = trie::BinaryTrieNode::build(&strs1_as_u8, &inds1);

    let strs2_as_u8: Vec<Vec<u8>> = strs2.iter().map(|s| s.as_bytes().to_vec()).collect();
    let inds2: Vec<Vec<(usize, usize)>> = (0..strs2.len()).map(|x| vec![(2, x)]).collect();
    let trie2 = trie::BinaryTrieNode::build(&strs2_as_u8, &inds2);

    let merged = trie::merge_tries(&trie1, &trie2);

    let queries = [
        "abcde", "abcdf", "abcdg", "two", "bbb", "bbbb", "bbbbb", "bbbbbb", "bbbbbbb", "abd",
        "abg", "!@#$", "a", "",
    ];

    for query in queries.iter() {
        let query_u8 = query.as_bytes().to_vec();
        let res = merged.query(&query_u8);

        // make sure behavior is correct: allow false positives, but no false negatives
        let actual_inds1 = strs1
            .iter()
            .enumerate()
            .filter(|(_, s)| s.eq(&query))
            .map(|(i, _)| (1, i))
            .collect::<Vec<(usize, usize)>>();
        let actual_inds2 = strs2
            .iter()
            .enumerate()
            .filter(|(_, s)| s.eq(&query))
            .map(|(i, _)| (2, i))
            .collect::<Vec<(usize, usize)>>();

        println!("query {}, res: {:?}", query, res);
        for i in actual_inds1.iter() {
            println!("query {}, checking {:?}", query, i);
            assert!(res.contains(i));
        }
        for i in actual_inds2.iter() {
            println!("query {}, checking {:?}", query, i);
            assert!(res.contains(i));
        }
    }
}

#[test]
fn extend_and_query() {
    let mut strs1 = [
        "abcde", "abcdg", "abcd", "two", "acbde", "bbbbb", "bbbbbb", "abd", "abe", "!@#$",
    ];
    strs1.sort();

    let mut strs2 = [
        "abcde", "abcdf", "ab", "two", "two", "bbbb", "bbbbbb", "abd", "abz", "",
    ];
    strs2.sort();

    let strs1_as_u8: Vec<Vec<u8>> = strs1.iter().map(|s| s.as_bytes().to_vec()).collect();
    let inds1: Vec<Vec<(usize, usize)>> = (0..strs1.len()).map(|x| vec![(1, x)]).collect();
    let mut trie1 = trie::BinaryTrieNode::build(&strs1_as_u8, &inds1);

    let strs2_as_u8: Vec<Vec<u8>> = strs2.iter().map(|s| s.as_bytes().to_vec()).collect();
    let inds2: Vec<Vec<(usize, usize)>> = (0..strs2.len()).map(|x| vec![(2, x)]).collect();
    let trie2 = trie::BinaryTrieNode::build(&strs2_as_u8, &inds2);

    trie1.extend(trie2);

    let queries = [
        "abcde", "abcdf", "abcdg", "two", "bbb", "bbbb", "bbbbb", "bbbbbb", "bbbbbbb", "abd",
        "abg", "!@#$", "a", "",
    ];

    for query in queries.iter() {
        let query_u8 = query.as_bytes().to_vec();
        let res = trie1.query(&query_u8);

        // make sure behavior is correct: allow false positives, but no false negatives
        let actual_inds1 = strs1
            .iter()
            .enumerate()
            .filter(|(_, s)| s.eq(&query))
            .map(|(i, _)| (1, i))
            .collect::<Vec<(usize, usize)>>();
        let actual_inds2 = strs2
            .iter()
            .enumerate()
            .filter(|(_, s)| s.eq(&query))
            .map(|(i, _)| (2, i))
            .collect::<Vec<(usize, usize)>>();

        println!("query {}, res: {:?}", query, res);
        for i in actual_inds1.iter() {
            println!("query {}, checking {:?}", query, i);
            assert!(res.contains(i));
        }
        for i in actual_inds2.iter() {
            println!("query {}, checking {:?}", query, i);
            assert!(res.contains(i));
        }
    }
}
