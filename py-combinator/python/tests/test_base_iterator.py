from collections.abc import Callable, Iterable
from copy import deepcopy
from itertools import islice
from typing import Any

import pytest
from py_combinator import (
    BaseIterator,
    DoubleEndedIterator,
    SizedDoubleEndedIterator,
    iterator_from,
)

# ruff: noqa: E731 S101

type IT_T = BaseIterator | DoubleEndedIterator | SizedDoubleEndedIterator


def create_base_iterator(data: Iterable[Any]) -> IT_T:
    """Create a BaseIterator from data."""
    return iterator_from(iter(data))


def create_sized_double_ended_iterator(data: Iterable[Any]) -> IT_T:
    """Create a SizedDoubleEndedIterator from data."""
    return iterator_from(data)


@pytest.mark.parametrize(
    ("iterator_creator", "expected_type"),
    [
        (create_base_iterator, BaseIterator),
        (create_sized_double_ended_iterator, SizedDoubleEndedIterator),
    ],
)
class TestBaseIterator:
    def test_map_addition(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        f = lambda x: x + 1
        lib_mapped = it.map(f).to_list()
        native_mapped = list(map(f, nums))
        assert lib_mapped == native_mapped

    def test_multiple_maps(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        f1 = lambda x: x + 1
        f2 = lambda x: x * 2
        lib_mapped = it.map(f1).map(f2).to_list()
        native_mapped = list(map(f2, map(f1, nums)))
        assert lib_mapped == native_mapped

    def test_list_conversion(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3]
        it = iterator_creator(nums)
        assert isinstance(it, expected_type)
        assert it.to_list() == nums

    def test_fold(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3]
        it = iterator_creator(nums)
        assert isinstance(it, expected_type)
        f = lambda acc, x: acc * x
        acc = 1
        libfolded = it.fold(acc, f)
        [acc := f(acc, x) for x in nums]  # type: ignore[no-untyped-call]
        assert acc == libfolded

    def test_map_fold(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        f_add = lambda x: x + 1
        f_fold = lambda acc, x: acc * x
        acc = 1
        libfolded = it.map(f_add).fold(acc, f_fold)
        [acc := f_fold(acc, x) for x in map(f_add, nums)]  # type: ignore[no-untyped-call]
        assert acc == libfolded

    def test_enumerate(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        lib_enumerate = it.enumerate().to_list()
        native_enumerate = list(enumerate(nums))
        assert lib_enumerate == native_enumerate

    def test_filter(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3, 4, 5]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        f = lambda x: x % 2 == 0
        lib_filter = it.filter(f)
        native_filter = list(filter(f, nums))
        assert lib_filter.to_list() == list(native_filter)

    def test_filter_consecutive(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [2, 4, 6, 1, 2, 3]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        f = lambda x: x % 2 == 0
        lib_filter = it.filter(f).to_list()
        native_filter = list(filter(f, nums))
        assert lib_filter == native_filter

    def test_filter_negative_twice(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3, 4, 5]
        it = iterator_creator(deepcopy(nums))
        assert isinstance(it, expected_type)
        f = lambda x: x < 0
        lib_filter = it.filter(f).filter(f).to_list()
        native_filter = list(filter(f, filter(f, nums)))
        assert lib_filter == native_filter

    def test_take(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3, 4, 5]
        lib_it = iterator_creator(deepcopy(nums))
        assert isinstance(lib_it, expected_type)
        native_it = iter(deepcopy(nums))
        lib_taken = lib_it.take(1)
        native_taken = islice(native_it, 1)
        assert lib_taken.to_list() == list(native_taken)
        assert lib_it.to_list() == list(native_it)

    def test_take_2(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3, 4, 5]
        lib_it = iterator_creator(deepcopy(nums))
        assert isinstance(lib_it, expected_type)
        native_it = iter(deepcopy(nums))
        lib_taken = lib_it.take(2).to_list()
        native_taken = list(islice(native_it, 2))
        assert lib_taken == native_taken
        assert lib_it.to_list() == list(native_it)

    def test_map_take(
        self,
        iterator_creator: Callable[[Iterable[Any]], IT_T],
        expected_type: IT_T,
    ) -> None:
        nums = [1, 2, 3, 4, 5]
        lib_it = iterator_creator(deepcopy(nums))
        assert isinstance(lib_it, expected_type)
        native_it = iter(deepcopy(nums))
        f = lambda x: x + 10
        lib_res = lib_it.map(f).take(3).to_list()
        native_res = list(islice(map(f, native_it), 3))
        assert lib_res == native_res
