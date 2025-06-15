from copy import deepcopy

from py_combinator import (
    SizedDoubleEndedIterator,
    iterator_from,
)

# ruff: noqa: E731 S101


class TestSizedDoubleEndedIterator:
    def test_reverse(self) -> None:
        nums = [1, 2, 3]
        it = iterator_from(deepcopy(nums))
        assert isinstance(it, SizedDoubleEndedIterator)
        lib_reversed = it.rev().to_list()
        native_reversed = list(reversed(nums))
        assert native_reversed == lib_reversed

    def test_map_reverse(self) -> None:
        nums = [1, 2, 3]
        it = iterator_from(deepcopy(nums))
        assert isinstance(it, SizedDoubleEndedIterator)
        f = lambda x: x + 1
        lib_reversed = it.map(f).rev().to_list()
        native_reversed = list(reversed(list(map(f, nums))))
        assert native_reversed == lib_reversed

    def test_enumerate_rev(self) -> None:
        nums = [1, 2, 3]
        it = iterator_from(deepcopy(nums))
        assert isinstance(it, SizedDoubleEndedIterator)
        assert it.enumerate().rev().to_list() == [(2, 3), (1, 2), (0, 1)]

    def test_rev_enumerate(self) -> None:
        nums = [1, 2, 3]
        it = iterator_from(deepcopy(nums))
        assert isinstance(it, SizedDoubleEndedIterator)
        assert it.rev().enumerate().to_list() == [(0, 3), (1, 2), (2, 1)]
