import py_combinator._py_combinator as rs
from py_combinator import AnyIterator

# ruff: noqa: E731 S101


def test_map_addition() -> None:
    nums = [1, 2, 3]
    it = AnyIterator(nums)
    f = lambda x: x + 1

    lib_mapped = it.map(f).to_list()
    native_mapped = list(map(f, nums))

    assert lib_mapped == native_mapped


def test_multiple_maps() -> None:
    nums = [1, 2, 3]
    it = AnyIterator(nums)
    f1 = lambda x: x + 1
    f2 = lambda x: x * 2

    lib_mapped = it.map(f1).map(f2).to_list()
    native_mapped = list(map(f2, map(f1, nums)))

    assert lib_mapped == native_mapped


def test_list_conversion() -> None:
    nums = [1, 2, 3]

    it = AnyIterator(nums)

    assert it.to_list() == nums


def test_iterator_consumption() -> None:
    nums = [1, 2, 3]

    it = AnyIterator(nums)

    it.to_list()

    assert len(it.to_list()) == 0


def test_chaining() -> None:
    c1 = rs.AnyIterator([1, 2, 3])
    c2 = c1.map(lambda x: x)
    c3 = c2.map(lambda x: x)

    assert c1 is c2 is c3


def test_uncalled_count() -> None:
    c1 = rs.AnyIterator([1, 2, 3])

    assert c1.uncalled == 0

    c1.map(lambda x: x)

    assert c1.uncalled == 1
