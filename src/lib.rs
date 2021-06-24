use std::{
    cmp,
    collections::{HashMap, HashSet},
};

#[derive(Debug, PartialEq)]
pub struct Suffix {
    item: usize,
    start: usize,
}

#[allow(unused)]
pub struct MatchDetails {
    len_1: usize,
    len_2: usize,
    len_overlap: usize,
    start_1: usize,
    start_2: usize,
    overlap_pct: f32,
}

fn get_longest_common_prefix(a: &str, b: &str) -> usize {
    let pairs = a.chars().zip(b.chars());
    pairs.take_while(|(a, b)| a == b).count()
}

#[derive(Debug)]
pub struct BaseGeneralizedSuffixArray {
    pub items: String,
    pub item_ranges: Vec<(usize, usize)>,
    pub suffixes: Vec<Suffix>,
    pub lcp_array: Vec<usize>,
}

#[allow(unused)]
impl BaseGeneralizedSuffixArray {
    pub fn new(items: &[&str]) -> Self {
        let mut suffixes = Vec::new();

        for (item, content) in items.iter().enumerate() {
            for (start, _) in content.char_indices() {
                suffixes.push(Suffix { item, start });
            }
        }
        suffixes.sort_by_key(|suffix| &items[suffix.item][suffix.start..]);

        // insert dummy empty strings to simplify looping logic
        let mut lcp_array = vec![0];
        for (a, b) in suffixes.iter().zip(suffixes.iter().skip(1)) {
            let s1 = &items[a.item][a.start..];
            let s2 = &items[b.item][b.start..];
            lcp_array.push(get_longest_common_prefix(s1, s2));
        }
        lcp_array.push(0);

        let (items, item_ranges) = Self::build_storage_data(&items);

        Self {
            items,
            item_ranges,
            suffixes,
            lcp_array,
        }
    }

    fn build_storage_data(items: &[&str]) -> (String, Vec<(usize, usize)>) {
        let mut storage_items = String::with_capacity(items.iter().map(|s| s.len()).sum());
        let mut item_ranges = Vec::with_capacity(items.len());
        let mut offset = 0;

        for item in items {
            storage_items.push_str(item);
            item_ranges.push((offset, offset + item.len()));
            offset += item.len();
        }

        (storage_items, item_ranges)
    }

    /// Get all suffixes around start_idx that share at least min_pcl elements with the query
    fn get_neighborhood(
        &self,
        query: &str,
        start_idx: usize,
        min_overlap_chars: usize,
    ) -> impl Iterator<Item = (usize, usize)> + '_ {
        let forward = {
            let mut pos = start_idx;
            let mut lcp = (start_idx < self.suffixes.len())
                .then(|| get_longest_common_prefix(&self.get_suffix(start_idx), query))
                .unwrap_or(0);

            std::iter::from_fn(move || {
                if pos < self.suffixes.len() && lcp >= min_overlap_chars {
                    let res = (pos, lcp);

                    lcp = cmp::min(lcp, self.lcp_array[pos + 1]);
                    pos += 1;

                    Some(res)
                } else {
                    None
                }
            })
        };

        let backward = {
            let mut pos = start_idx;
            let mut lcp = (pos > 0)
                .then(|| get_longest_common_prefix(&self.get_suffix(pos - 1), query))
                .unwrap_or(0);

            std::iter::from_fn(move || {
                if pos > 0 && lcp >= min_overlap_chars {
                    let res = (pos - 1, lcp);

                    lcp = cmp::min(lcp, self.lcp_array[pos - 1]);
                    pos -= 1;

                    Some(res)
                } else {
                    None
                }
            })
        };

        forward.chain(backward)
    }

    /// get all items for which the longest common substring with the query has length at least min_pcl
    pub fn similar(
        &self,
        query: &str,
        min_overlap_chars: usize,
        min_overlap_pct: f32,
    ) -> HashMap<usize, MatchDetails> {
        fn prev_is_larger(prev: &MatchDetails, new: &MatchDetails) -> bool {
            (prev.len_overlap > new.len_overlap) || (prev.start_1 < new.start_1)
        }

        let mut res: HashMap<usize, MatchDetails> = HashMap::new();

        let len = (query.chars().count() + 1).saturating_sub(min_overlap_chars);

        for (charidx, (offset, _)) in query.char_indices().enumerate() {
            if charidx >= len {
                break;
            }
            let q = &query[offset..];
            let (Ok(start_idx) | Err(start_idx)) = self
                .suffixes
                .binary_search_by(|probe| self.get_item_suffix(probe).cmp(q));

            for (idx, len_overlap) in self.get_neighborhood(q, start_idx, min_overlap_chars) {
                let suffix = &self.suffixes[idx];

                let len_1 = query.chars().count();
                let len_2 = self.get_item_unchecked(suffix.item).chars().count();
                let start_1 = offset;
                let start_2 = suffix.start;
                let overlap_pct = (2. * len_overlap as f32) / ((len_1 + len_2) as f32);

                if overlap_pct < min_overlap_pct {
                    continue;
                }

                let match_details = MatchDetails {
                    len_1,
                    len_2,
                    len_overlap,
                    start_1,
                    start_2,
                    overlap_pct,
                };

                match res.get(&suffix.item) {
                    // keep the old entry
                    Some(prev) if prev_is_larger(prev, &match_details) => {}
                    // upsert the entry
                    _ => {
                        res.insert(suffix.item, match_details);
                    }
                }
            }
        }
        res
    }

    /// Helper function to use strings as inputs and outputs
    pub fn similar_str(
        &self,
        query: &str,
        min_overlap_chars: usize,
        min_overlap_pct: f32,
    ) -> HashSet<String> {
        let res = self.similar(&query, min_overlap_chars, min_overlap_pct);

        res.keys()
            .map(|&i| self.get_item_unchecked(i).to_owned())
            .collect()
    }

    fn get_suffix(&self, suffix_idx: usize) -> &str {
        self.get_item_suffix(&self.suffixes[suffix_idx])
    }

    fn get_item_suffix(&self, suffix: &Suffix) -> &str {
        let (start, end) = self.item_ranges[suffix.item];
        &self.items[start..end][suffix.start..]
    }

    fn get_item(&self, idx: usize) -> Option<&str> {
        let &(start, end) = self.item_ranges.get(idx)?;
        Some(&self.items[start..end])
    }

    fn get_item_unchecked(&self, idx: usize) -> &str {
        let (start, end) = self.item_ranges[idx];
        &self.items[start..end]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test that the suffix array is correctly constructed
    ///
    #[test]
    fn correct_construction() {
        let index = BaseGeneralizedSuffixArray::new(&["hello", "bella"]);
        println!("{:?}", index);

        assert_eq!(index.items, "hellobella");
        assert_eq!(index.item_ranges, vec![(0, 5), (5, 10)]);

        assert_eq!(
            index.suffixes,
            vec![
                Suffix { item: 1, start: 4 }, // "a",
                Suffix { item: 1, start: 0 }, // "bella",
                Suffix { item: 1, start: 1 }, // "ella",
                Suffix { item: 0, start: 1 }, // "ello",
                Suffix { item: 0, start: 0 }, // "hello",
                Suffix { item: 1, start: 3 }, // "la",
                Suffix { item: 1, start: 2 }, // "lla",
                Suffix { item: 0, start: 2 }, // "llo",
                Suffix { item: 0, start: 3 }, // "lo",
                Suffix { item: 0, start: 4 }, // "o",
            ]
        );

        assert_eq!(
            index.lcp_array,
            vec![
                0, // $, "a",
                0, // "a", "bella",
                0, // "bella", "ella",
                3, // "ella", "ello",
                0, // "ello", "hello",
                0, // "hello", "la",
                1, // "la", "lla",
                2, // "lla", "llo",
                1, // "llo", "lo",
                0, // "lo", "o",
                0, // "o", $
            ],
        );
    }

    /// Test some example queries
    ///
    #[test]
    fn queries() {
        fn stringset(items: &[&str]) -> HashSet<String> {
            items.iter().map(|&s| s.to_owned()).collect()
        }

        let index = BaseGeneralizedSuffixArray::new(&["hello", "bella"]);
        println!("{:?}", index);

        let actual = index.similar_str("illi", 2, 0.0);
        let expected = stringset(&["hello", "bella"]);

        assert_eq!(actual, expected);

        let actual = index.similar_str("illi", 3, 0.0);
        let expected = stringset(&[]);

        assert_eq!(actual, expected);

        let actual = index.similar_str("illo", 3, 0.0);
        let expected = stringset(&["hello"]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn ensure_empty_index_does_not_panic() {
        let index = BaseGeneralizedSuffixArray::new(&[]);
        println!("{:?}", index);

        index.similar_str("illi", 2, 0.0);
    }

    /// Test some example queries
    ///
    #[test]
    fn utf8_queries() {
        fn stringset(items: &[&str]) -> HashSet<String> {
            items.iter().map(|&s| s.to_owned()).collect()
        }

        let index = BaseGeneralizedSuffixArray::new(&["hällö", "bällä"]);
        println!("{:?}", index);

        let actual = index.similar_str("illi", 2, 0.0);
        let expected = stringset(&["hällö", "bällä"]);
        assert_eq!(actual, expected);

        let actual = index.similar_str("illi", 3, 0.0);
        let expected = stringset(&[]);
        assert_eq!(actual, expected);

        let actual = index.similar_str("illö", 3, 0.0);
        let expected = stringset(&["hällö"]);
        assert_eq!(actual, expected);

        let actual = index.similar_str("ällö", 3, 0.0);
        let expected = stringset(&["hällö", "bällä"]);
        assert_eq!(actual, expected);
    }
}

/// The Python wrapper
///
#[cfg(not(test))]
mod py {
    use pyo3::{
        class::PySequenceProtocol,
        exceptions::{PyIndexError, PyValueError},
        prelude::*,
    };
    use std::convert::{TryFrom, TryInto};

    use super::*;

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
        overlap_pct: f32,
    }

    /// Convert the internal MatchDetails struct into the one exposed to Python
    impl From<super::MatchDetails> for MatchDetails {
        fn from(md: super::MatchDetails) -> MatchDetails {
            MatchDetails {
                len_1: md.len_1,
                len_2: md.len_2,
                len_overlap: md.len_overlap,
                start_1: md.start_1,
                start_2: md.start_2,
                overlap_pct: md.overlap_pct,
            }
        }
    }

    #[pyclass]
    #[text_signature = "(strings)"]
    pub struct GeneralizedSuffixArray {
        suffix_array: BaseGeneralizedSuffixArray,
    }

    #[pymethods]
    impl GeneralizedSuffixArray {
        #[new]
        pub fn new(items: Vec<&str>) -> Self {
            Self {
                suffix_array: BaseGeneralizedSuffixArray::new(&items),
            }
        }

        #[text_signature = "(query, min_overlap_chars, min_overlap_pct)"]
        pub fn similar(
            &self,
            query: &str,
            min_overlap_chars: Option<usize>,
            min_overlap_pct: Option<f32>,
        ) -> PyResult<HashMap<usize, MatchDetails>> {
            let min_pct = min_overlap_pct.unwrap_or(0.0);

            let min_chars_from_pct = min_pct * (query.chars().count() as f32) / (2. - min_pct);
            let min_chars_from_pct: usize = (min_chars_from_pct.ceil() as i64)
                .try_into()
                .map_err(|_| PyValueError::new_err("Invalid values for min_overlap."))?;

            let min_chars = match (min_overlap_chars, min_overlap_pct) {
                (Some(val), _) => val,
                (_, Some(_)) => 0,
                _ => return Err(PyValueError::new_err("Invalid values for min_overlap.")),
            };

            let min_chars = cmp::max(min_chars, min_chars_from_pct);

            let res = self
                .suffix_array
                .similar(&query, min_chars, min_pct)
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<HashMap<_, _>>();

            Ok(res)
        }
    }

    #[pyproto]
    impl PySequenceProtocol for GeneralizedSuffixArray {
        fn __len__(&self) -> usize {
            self.suffix_array.items.len()
        }

        fn __getitem__(&self, key: isize) -> PyResult<String> {
            let idx = usize::try_from(key)?;
            let res = self
                .suffix_array
                .get_item(idx)
                .ok_or_else(|| PyIndexError::new_err(()))?;
            Ok(res.to_owned())
        }
    }

    #[pymodule]
    fn generalized_suffix_array(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_class::<GeneralizedSuffixArray>()?;
        m.add_class::<MatchDetails>()?;
        Ok(())
    }
}
