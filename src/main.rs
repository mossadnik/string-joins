use std::fs;
use std::ops;
use std::cmp;
use std::collections::HashMap;


struct Suffix {
    item: usize,
    start: usize
}

type ItemType = Vec<char>;
type SliceType = [char];


#[derive(Debug)]
struct SimilarItem {
    item_idx: usize,
    pcl: usize
}


struct GeneralizedSuffixArray {
    pub items: Vec<ItemType>,
    pub suffixes: Vec<Suffix>,
    pub lcp_array: Vec<usize>
}


fn get_longest_common_prefix(a: &SliceType, b: &SliceType) -> usize {
    let mut res: usize = 0;
    for (c1, c2) in a.iter().zip(b.iter()) {
        if c1 == c2 {
            res += 1;
        } else {
            break;
        }
    }
    let s1: String = a.iter().collect();
    let s2: String = b.iter().collect();
    println!("{}  {}", s1, s2);
    res
}

fn get_item_suffix<'a>(items: &'a Vec<ItemType>, suffix: &Suffix) -> &'a SliceType {
    &items[suffix.item][suffix.start..]
}

// TODO: Use hashmap to aggregate results

impl GeneralizedSuffixArray {
    pub fn new(items: Vec<ItemType>) -> Self {

        let mut suffixes: Vec<Suffix> = vec![];

        for (item, content) in items.iter().enumerate() {
            for start in 0..content.len() {
                suffixes.push(Suffix {item, start});
            }
        }
        suffixes.sort_by_key(|suffix| &items[suffix.item][suffix.start..]);

        let mut lcp_array: Vec<usize> = vec![];
        for (a, b) in suffixes.iter().zip(suffixes.iter().skip(1)) {
            let s1 = get_item_suffix(&items, a);
            let s2 = get_item_suffix(&items, b);
            lcp_array.push(get_longest_common_prefix(s1, s2));
        }
        lcp_array.push(0);

        Self { items, suffixes, lcp_array }
    }

    fn get_neighborhood(&self, query: &SliceType, start_idx: usize, min_pcl: usize) -> HashMap<usize, usize> {
        let mut res: HashMap<usize, usize> = HashMap::new();
        if start_idx < self.suffixes.len() {
            let mut pcl = get_longest_common_prefix(&self[start_idx], query);
            if pcl >= min_pcl {
                res.insert(self.suffixes[start_idx].item, pcl);

                for i in start_idx..self.suffixes.len() - 1 {
                    pcl = cmp::min(pcl, self.lcp_array[i]);
                    if pcl >= min_pcl {
                        res.insert(self.suffixes[i + 1].item, pcl);
                    } else {
                        break;
                    }
                }
            }
        }

        if start_idx > 0 {
            let mut pcl = get_longest_common_prefix(&self[start_idx - 1], query);
            if pcl >= min_pcl {
                res.insert(self.suffixes[start_idx - 1].item, pcl);

                for i in (0..start_idx - 1).rev() {
                    pcl = cmp::min(pcl, self.lcp_array[i]);
                    if pcl >= min_pcl {
                        res.insert(self.suffixes[i].item, pcl);
                    } else {
                        break;
                    }
                }
            }
        }
        res

    }

    pub fn similar(&self, query: &SliceType, min_pcl: usize) -> HashMap<usize, usize> {
        let mut res: HashMap<usize, usize> = HashMap::new();

        for offset in 0..query.len() - min_pcl + 1 {
            let q = &query[offset..];
            let start_idx = self.suffixes.binary_search_by(|probe| get_item_suffix(&self.items, probe).cmp(q));
            let start_idx = match start_idx {
                Ok(idx) => idx,
                Err(idx) => idx
            };

            for (item_idx, pcl) in self.get_neighborhood(q, start_idx, min_pcl).iter() {
                let current_pcl = res.entry(*item_idx).or_insert(0);
                *current_pcl = cmp::max(*current_pcl, *pcl);
            }
        }

        res
    }

}

impl ops::Index<usize> for GeneralizedSuffixArray {
    type Output = SliceType;

    fn index(&self, idx: usize) -> &Self::Output {
        let suffix = &self.suffixes[idx];
        get_item_suffix(&self.items, suffix)
    }
}


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
    let vecs: Vec<ItemType> = lines
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
