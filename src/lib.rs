use std::ops;
use std::cmp;
use std::isize;
use std::convert::TryFrom;
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::class::{
    PySequenceProtocol,
};
use pyo3::exceptions::{PyIndexError, PyValueError};


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
    fn get_neighborhood(&self, query: &SliceType, start_idx: usize, min_overlap_chars: usize, min_overlap_pct: f32) -> Vec<(&Suffix, usize)> {

        let mut res: Vec<(&Suffix, usize)> = Vec::new();

        let check_overlap_pct = |overlap: usize, len1: usize, len2: usize| -> bool {
            (2. * overlap as f32) / ((len1 + len2) as f32) >= min_overlap_pct
        };

        let mut insert_result = |idx: usize, pcl: usize| -> () {
            let suffix = &self.suffixes[idx];
            // check overlap pct
            if min_overlap_pct == 0.0 || check_overlap_pct(pcl, query.len(), self.items[suffix.item].len()) {
                res.push((suffix, pcl));
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
    pub fn similar(&self, query: &SliceType, min_overlap_chars: usize, min_overlap_pct: f32) -> HashMap<usize, usize> {
        let mut res: HashMap<usize, usize> = HashMap::new();

        let len = query.len() + 1;
        let len = if len > min_overlap_chars {len - min_overlap_chars} else {0};

        for offset in 0..len {
            let q = &query[offset..];
            let start_idx = self.suffixes.binary_search_by(|probe| get_item_suffix(&self.items, probe).cmp(q));
            let start_idx = match start_idx {
                Ok(idx) => idx,
                Err(idx) => idx
            };

            for (suffix, overlap_chars) in self.get_neighborhood(q, start_idx, min_overlap_chars, min_overlap_pct).iter() {
                let current_overlap_chars = res.entry(suffix.item).or_insert(0);
                *current_overlap_chars = cmp::max(*current_overlap_chars, *overlap_chars);
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


/// GeneralizedSuffixArray(strings: List[str])
/// --
///
#[pyclass]
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

    pub fn similar(&self, query: &str, min_overlap_chars: Option<usize>, min_overlap_pct: Option<f32>) -> PyResult<HashMap<usize, usize>> {
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
    Ok(())
}
