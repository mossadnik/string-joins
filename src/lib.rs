use std::ops;
use std::cmp;
use std::isize;
use std::convert::TryFrom;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use pyo3::prelude::*;
use pyo3::class::{
    PySequenceProtocol,
};
use pyo3::exceptions::{PyIndexError, PyValueError};


pub struct Suffix {
    item: usize,
    start: usize
}


#[pyclass]
pub struct MatchDetails {
    #[pyo3(get)]
    len_1: usize,
    #[pyo3(get)]
    len_2: usize,
    #[pyo3(get)]
    len_overlap: usize,
    #[pyo3(get)]
    start_1: usize,
    #[pyo3(get)]
    start_2: usize,
    #[pyo3(get)]
    overlap_pct: f32
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


pub struct BaseGeneralizedSuffixArray {
    pub items: Vec<ItemType>,
    pub suffixes: Vec<Suffix>,
    pub lcp_array: Vec<usize>
}


impl BaseGeneralizedSuffixArray {
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
    fn get_neighborhood(&self, query: &SliceType, start_idx: usize, min_overlap_chars: usize, min_overlap_pct: f32) -> Vec<(&Suffix, MatchDetails)> {

        let mut res: Vec<(&Suffix, MatchDetails)> = Vec::new();

        let mut insert_result = |idx: usize, len_overlap: usize| -> () {
            let suffix = &self.suffixes[idx];
            // check overlap pct
            let len_1 = query.len();
            let len_2 = self.items[suffix.item].len();
            let overlap_pct = (2. * len_overlap as f32) / ((len_1 + len_2) as f32);

            if overlap_pct >= min_overlap_pct {
                res.push((suffix, MatchDetails{ len_1, len_2, len_overlap, start_1: 0, start_2: 0, overlap_pct }));
            }
        };
        if start_idx < self.suffixes.len() {
            let mut lcp = get_longest_common_prefix(&self[start_idx], query);
            if lcp >= min_overlap_chars {
                insert_result(start_idx, lcp);
                for i in start_idx..self.suffixes.len() - 1 {
                    lcp = cmp::min(lcp, self.lcp_array[i]);
                    if lcp >= min_overlap_chars {
                        insert_result(i + 1, lcp);
                    } else {
                        break;
                    }
                }
            }
        }

        if start_idx > 0 {
            let mut lcp = get_longest_common_prefix(&self[start_idx - 1], query);
            if lcp >= min_overlap_chars {
                insert_result(start_idx - 1, lcp);
                for i in (0..start_idx - 1).rev() {
                    lcp = cmp::min(lcp, self.lcp_array[i]);
                    if lcp >= min_overlap_chars {
                        insert_result(i, lcp);
                    } else {
                        break;
                    }
                }
            }
        }
        res
    }

    /// get all items for which the longest common substring with the query has length at least min_pcl
    pub fn similar(&self, query: &SliceType, min_overlap_chars: usize, min_overlap_pct: f32) -> HashMap<usize, MatchDetails> {
        let mut res: HashMap<usize, MatchDetails> = HashMap::new();

        let len = query.len() + 1;
        let len = if len > min_overlap_chars {len - min_overlap_chars} else {0};

        for offset in 0..len {
            let q = &query[offset..];
            let start_idx = self.suffixes.binary_search_by(|probe| get_item_suffix(&self.items, probe).cmp(q));
            let start_idx = match start_idx {
                Ok(idx) => idx,
                Err(idx) => idx
            };

            for (suffix, mut match_details) in self.get_neighborhood(q, start_idx, min_overlap_chars, min_overlap_pct).into_iter() {
                let entry = res.entry(suffix.item);
                match_details.start_1 = offset;
                match_details.start_2 = suffix.start;
                match entry {
                    Entry::Vacant(e) => {e.insert(match_details);},
                    Entry::Occupied(mut e) => {
                        let current_match_details = e.get();
                        if match_details.len_overlap >= current_match_details.len_overlap
                        && match_details.start_1 <= current_match_details.start_1 {
                            e.insert(match_details);
                        }
                    }
                };
            }
        }
        res
    }
}


impl ops::Index<usize> for BaseGeneralizedSuffixArray {
    type Output = SliceType;

    fn index(&self, idx: usize) -> &Self::Output {
        let suffix = &self.suffixes[idx];
        get_item_suffix(&self.items, suffix)
    }
}


#[pyclass]
#[text_signature = "(strings)"]
pub struct GeneralizedSuffixArray {
    suffix_array: BaseGeneralizedSuffixArray
}

impl GeneralizedSuffixArray {
    fn get_item(&self, idx: usize) -> Result<String, ()> {
        if idx < self.suffix_array.items.len() {
            Ok(self.suffix_array.items[idx].iter().collect::<String>())
        } else {
            Err(())
        }
    }
}


#[pymethods]
impl GeneralizedSuffixArray {
    #[new]
    pub fn new(strings: Vec<&str>) -> Self {
        let items: Vec<Vec<char>> = strings
        .into_iter()
        .map(|line| line.chars().collect())
        .collect();

        Self { suffix_array: BaseGeneralizedSuffixArray::new(items) }
    }

    #[text_signature = "(query, min_overlap_chars, min_overlap_pct)"]
    pub fn similar(
        &self, query: &str,
        min_overlap_chars: Option<usize>,
        min_overlap_pct: Option<f32>
    ) -> PyResult<HashMap<usize, MatchDetails>> {
        let min_pct = match min_overlap_pct {
            Some(val) => val,
            _ => 0.0
        };

        let min_chars_from_pct = usize::try_from(
            (min_pct * (query.len() as f32) / (2. - min_pct)).ceil() as i64
        );
        let min_chars_from_pct = match min_chars_from_pct {
            Ok(val) => val,
            _ => return Err(PyValueError::new_err("Invalid values for min_overlap."))
        };

        let min_chars = match min_overlap_chars {
            Some(val) => val,
            _ => match min_overlap_pct {
                Some(_) => 0,
                _ => return Err(PyValueError::new_err("Invalid values for min_overlap."))
            }
        };
        let min_chars = cmp::max(min_chars, min_chars_from_pct);

        let q: Vec<char> = query.chars().collect();
        Ok(self.suffix_array.similar(&q, min_chars, min_pct))
    }

}

#[pyproto]
impl PySequenceProtocol for GeneralizedSuffixArray {
    fn __len__(&self) -> usize {
        self.suffix_array.items.len()
    }

    fn __getitem__(&self, key: isize) -> PyResult<String> {
        let idx = usize::try_from(key)?;
        let res = self.get_item(idx);
        match res {
            Ok(s) => Ok(s),
            Err(_) => Err(PyIndexError::new_err(()))
        }
    }
}


#[pymodule]
fn generalized_suffix_array(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GeneralizedSuffixArray>()?;
    m.add_class::<MatchDetails>()?;
    Ok(())
}
