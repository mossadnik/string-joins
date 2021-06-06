import pytest

from generalized_suffix_array import GeneralizedSuffixArray


@pytest.mark.parametrize('query, min_overlap_chars, min_overlap_pct, expected', [
    ('ell', 3, None, {0: 3, 1: 3}),
    ('hell', 3, None, {0: 4, 1: 3}),
    ('yello', 4, None, {0: 4}),
    ('xyz', 1, None, {}),
    ('hell', None, .8, {0: 4}),
    ('ell', 3, .1, {0: 3, 1: 3}),
    ('all', 3, .1, {})  # min_overlap_chars prevents match
])
def test_similar(query, min_overlap_chars, min_overlap_pct, expected):
    strings = ['hello', 'bella']

    index = GeneralizedSuffixArray(strings)

    actual = index.similar(query, min_overlap_chars, min_overlap_pct)
    assert actual == expected


def test_similar_no_overlap_raises():
    strings = ['hello', 'bello']
    index = GeneralizedSuffixArray(strings)

    with pytest.raises(ValueError):
        index.similar('abc', None, None)


def test_dunder_getitem():
    strings = ['hello', 'bello']
    index = GeneralizedSuffixArray(strings)

    for i, s in enumerate(strings):
        assert index[i] == s
