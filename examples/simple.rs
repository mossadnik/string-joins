use std::fs;
extern crate generalized_suffix_array;
use generalized_suffix_array::StringGeneralizedSuffixArray;


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

    for vec in lines.iter() {
        println!("{}", vec);
    }
    println!("");

    let suffix_array = StringGeneralizedSuffixArray::new(lines);


    let query = "seel";
    for (item_idx, pcl) in suffix_array.similar(&query, 3).iter() {
        let s: String = suffix_array.get_item(*item_idx);
        println!("{} {}", *pcl, s);
    }
}
