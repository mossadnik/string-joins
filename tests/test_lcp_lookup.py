import pytest

from generalized_suffix_array import GeneralizedSuffixArray


@pytest.mark.parametrize('query, min_lcp, expected', [
    ('ell', 3, {0: 3, 1: 3}),
    ('hell', 3, {0: 4, 1: 3}),
    ('yello', 4, {0: 4, 1: 4}),
    ('xyz', 1, {})
])
def test_lookup(query, min_lcp, expected):
    strings = ['hello', 'bello']

    index = GeneralizedSuffixArray(strings)

    actual = index.similar(query, min_lcp)
    assert actual == expected


def test_dunder_getitem():
    strings = ['hello', 'bello']
    index = GeneralizedSuffixArray(strings)

    for i, s in enumerate(strings):
        assert index[i] == s
