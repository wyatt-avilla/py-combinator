import py_combinator._py_combinator as rs
from py_combinator import IteratorOfInt


def test_map_addition() -> None:
    it = IteratorOfInt([1, 2, 3])
    mapped = it.map(lambda x: x + 1).go()
    assert list(mapped) == [2, 3, 4]


def test_chaining() -> None:
    c1 = rs.IteratorOfInt([1, 2, 3])
    c2 = c1.map(lambda x: x)
    c3 = c2.map(lambda x: x)

    assert c1 is c2 is c3


def test_uncalled_count() -> None:
    c1 = rs.IteratorOfInt([1, 2, 3])

    assert c1.uncalled == 0

    c1.map(lambda x: x)

    assert c1.uncalled == 1
