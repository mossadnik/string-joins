use std::{
    cmp,
    collections::{HashMap, HashSet},
    ops,
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

type ItemType = Vec<char>;
type SliceType = [char];

fn get_longest_common_prefix(a: &SliceType, b: &SliceType) -> usize {
    let pairs = a.iter().zip(b.iter());
    pairs.take_while(|(a, b)| a == b).count()
}

fn get_item_suffix<'a>(items: &'a [ItemType], suffix: &Suffix) -> &'a SliceType {
    &items[suffix.item][suffix.start..]
}

#[derive(Debug)]
pub struct BaseGeneralizedSuffixArray {
    pub items: Vec<ItemType>,
    pub suffixes: Vec<Suffix>,
    pub lcp_array: Vec<usize>,
}

impl BaseGeneralizedSuffixArray {
    pub fn new(items: Vec<ItemType>) -> Self {
        let mut suffixes = Vec::new();

        for (item, content) in items.iter().enumerate() {
            for start in 0..content.len() {
                suffixes.push(Suffix { item, start });
            }
        }
        suffixes.sort_by_key(|suffix| &items[suffix.item][suffix.start..]);

        let mut lcp_array = Vec::new();
        for (a, b) in suffixes.iter().zip(suffixes.iter().skip(1)) {
            let s1 = get_item_suffix(&items, a);
            let s2 = get_item_suffix(&items, b);
            lcp_array.push(get_longest_common_prefix(s1, s2));
        }
        lcp_array.push(0);

        Self {
            items,
            suffixes,
            lcp_array,
        }
    }

    /// Get all suffixes around start_idx that share at least min_pcl elements with the query
    fn get_neighborhood(
        &self,
        query: &SliceType,
        start_idx: usize,
        min_overlap_chars: usize,
        min_overlap_pct: f32,
    ) -> Vec<(&Suffix, MatchDetails)> {
        let mut res: Vec<(&Suffix, MatchDetails)> = Vec::new();

        let mut insert_result = |idx: usize, len_overlap: usize| {
            let suffix = &self.suffixes[idx];
            // check overlap pct
            let len_1 = query.len();
            let len_2 = self.items[suffix.item].len();
            let overlap_pct = (2. * len_overlap as f32) / ((len_1 + len_2) as f32);

            if overlap_pct >= min_overlap_pct {
                res.push((
                    suffix,
                    MatchDetails {
                        len_1,
                        len_2,
                        len_overlap,
                        start_1: 0,
                        start_2: 0,
                        overlap_pct,
                    },
                ));
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
    pub fn similar(
        &self,
        query: &SliceType,
        min_overlap_chars: usize,
        min_overlap_pct: f32,
    ) -> HashMap<usize, MatchDetails> {
        fn prev_is_larger(prev: &MatchDetails, new: &MatchDetails) -> bool {
            (prev.len_overlap > new.len_overlap) || (prev.start_1 < new.start_1)
        }

        let mut res: HashMap<usize, MatchDetails> = HashMap::new();

        let len = (query.len() + 1).saturating_sub(min_overlap_chars);

        for offset in 0..len {
            let q = &query[offset..];
            let (Ok(start_idx) | Err(start_idx)) = self
                .suffixes
                .binary_search_by(|probe| get_item_suffix(&self.items, probe).cmp(q));

            for (suffix, mut match_details) in
                self.get_neighborhood(q, start_idx, min_overlap_chars, min_overlap_pct)
            {
                match_details.start_1 = offset;
                match_details.start_2 = suffix.start;

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
        let query = query.chars().collect::<Vec<_>>();
        let res = self.similar(&query, min_overlap_chars, min_overlap_pct);

        res.keys()
            .map(|&i| self.items[i].iter().collect::<String>())
            .collect()
    }
}

impl ops::Index<usize> for BaseGeneralizedSuffixArray {
    type Output = SliceType;

    fn index(&self, idx: usize) -> &Self::Output {
        let suffix = &self.suffixes[idx];
        get_item_suffix(&self.items, suffix)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test that the suffix array is correctly constructed
    ///
    #[test]
    fn correct_construction() {
        let strings: Vec<Vec<char>> = vec!["hello".chars().collect(), "bella".chars().collect()];
        let index = BaseGeneralizedSuffixArray::new(strings);
        println!("{:?}", index);

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

        let strings: Vec<Vec<char>> = vec!["hello".chars().collect(), "bella".chars().collect()];
        let index = BaseGeneralizedSuffixArray::new(strings);
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

    impl GeneralizedSuffixArray {
        fn get_item(&self, idx: usize) -> Option<String> {
            self.suffix_array
                .items
                .get(idx)
                .map(|it| it.iter().collect::<String>())
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

            Self {
                suffix_array: BaseGeneralizedSuffixArray::new(items),
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

            let min_chars_from_pct = min_pct * (query.len() as f32) / (2. - min_pct);
            let min_chars_from_pct: usize = (min_chars_from_pct.ceil() as i64)
                .try_into()
                .map_err(|_| PyValueError::new_err("Invalid values for min_overlap."))?;

            let min_chars = match (min_overlap_chars, min_overlap_pct) {
                (Some(val), _) => val,
                (_, Some(_)) => 0,
                _ => return Err(PyValueError::new_err("Invalid values for min_overlap.")),
            };

            let min_chars = cmp::max(min_chars, min_chars_from_pct);

            let q: Vec<char> = query.chars().collect();
            let res = self
                .suffix_array
                .similar(&q, min_chars, min_pct)
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
                .get_item(idx)
                .ok_or_else(|| PyIndexError::new_err(()))?;
            Ok(res)
        }
    }

    #[pymodule]
    fn generalized_suffix_array(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_class::<GeneralizedSuffixArray>()?;
        m.add_class::<MatchDetails>()?;
        Ok(())
    }
}
