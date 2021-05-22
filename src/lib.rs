use pyo3::prelude::*;
use std::ops;
use std::cmp;
use std::collections::HashMap;


pub struct Suffix {
    item: usize,
    start: usize
}


type ItemType = Vec<char>;
type SliceType = [char];  // array


#[derive(Debug)]
struct SimilarItem {
    item_idx: usize,
    pcl: usize
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
    res
}


fn get_item_suffix<'a>(items: &'a Vec<ItemType>, suffix: &Suffix) -> &'a SliceType {
    &items[suffix.item][suffix.start..]
}


pub struct GeneralizedSuffixArray {
    pub items: Vec<ItemType>,
    pub suffixes: Vec<Suffix>,
    pub lcp_array: Vec<usize>
}


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

    /// Get all suffixes around start_idx that share at least min_pcl elemetns with the query
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

    /// get all items for which the longest common substring with the query has length at least min_pcl
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

#[pyclass]
pub struct StringGeneralizedSuffixArray {
    suffix_array: GeneralizedSuffixArray
}

#[pymethods]
impl StringGeneralizedSuffixArray {
    #[new]
    pub fn new(strings: Vec<&str>) -> Self {
        let items: Vec<Vec<char>> = strings
        .into_iter()
        .map(|line| line.chars().collect())
        .collect();

        Self { suffix_array: GeneralizedSuffixArray::new(items) }
    }

    pub fn similar(&self, query: &str, min_pcl: usize) -> HashMap<usize, usize> {
        let q: Vec<char> = query.chars().collect();
        self.suffix_array.similar(&q, min_pcl)
    }

    pub fn get_item(&self, idx: usize) -> String {
        let res: String = self.suffix_array.items[idx].iter().collect();
        res
    }
}


#[pymodule]
fn generalized_suffix_array(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<StringGeneralizedSuffixArray>()?;
    Ok(())
}
