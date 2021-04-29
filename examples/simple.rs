use std::fs;
extern crate generalized_suffix_array;
use generalized_suffix_array::GeneralizedSuffixArray;


fn main() {
    let filename = "./data/strings.txt";
    // TODO nicer to read lines directly
    let contents = fs::read_to_string(filename)
        .expect("error reading file");

    let lines: Vec<&str> = contents
        .split('\n')
        .map(str::trim)
        .filter(|line| line.len() > 0)
        .collect();

    // handle unicode better?
    let vecs: Vec<Vec<char>> = lines
        .into_iter()
        .map(|line| line.chars().collect())
        .collect();

    for vec in vecs.iter() {
        println!("{:?}", vec);
    }

    let suffix_array = GeneralizedSuffixArray::new(vecs);

    for (i, _) in suffix_array.suffixes.iter().enumerate() {
        let lcp = suffix_array.lcp_array[i];
        let s: String = suffix_array[i].iter().collect();
        println!("{} {} {}", i, lcp, s);
    }

    let query = vec!['s', 'e', 'e', 'l'];
    for (item_idx, pcl) in suffix_array.similar(&query, 3).iter() {
        let s: String = suffix_array.items[*item_idx].iter().collect();
        println!("{} {}", *pcl, s);
    }
}
