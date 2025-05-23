from copy import deepcopy
from itertools import islice

from py_combinator import iterator_from

# ruff: noqa: E731 S101


def test_map_addition() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))
    f = lambda x: x + 1

    lib_mapped = it.map(f).to_list()
    native_mapped = list(map(f, nums))

    assert lib_mapped == native_mapped


def test_multiple_maps() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))
    f1 = lambda x: x + 1
    f2 = lambda x: x * 2

    lib_mapped = it.map(f1).map(f2).to_list()
    native_mapped = list(map(f2, map(f1, nums)))

    assert lib_mapped == native_mapped


def test_list_conversion() -> None:
    nums = [1, 2, 3]

    it = iterator_from(nums)

    assert it.to_list() == nums


def test_fold() -> None:
    nums = [1, 2, 3]
    it = iterator_from(nums)

    f = lambda acc, x: acc * x
    acc = 1

    libfolded = it.fold(acc, f)
    [acc := f(acc, x) for x in nums]

    assert acc == libfolded


def test_map_fold() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))

    f_add = lambda x: x + 1
    f_fold = lambda acc, x: acc * x
    acc = 1

    libfolded = it.map(f_add).fold(acc, f_fold)
    [acc := f_fold(acc, x) for x in map(f_add, nums)]

    assert acc == libfolded


def test_reverse() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))

    lib_reversed = it.rev().to_list()
    native_reversed = list(reversed(nums))

    assert native_reversed == lib_reversed


def test_map_reverse() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))

    f = lambda x: x + 1

    lib_reversed = it.map(f).rev().to_list()
    native_reversed = list(reversed(list(map(f, nums))))

    assert native_reversed == lib_reversed


def test_enumerate() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))

    lib_enumerate = it.enumerate().to_list()
    native_enumerate = list(enumerate(nums))

    assert lib_enumerate == native_enumerate


def test_enumerate_rev() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))

    assert it.enumerate().rev().to_list() == [(2, 3), (1, 2), (0, 1)]


def test_rev_enumerate() -> None:
    nums = [1, 2, 3]
    it = iterator_from(deepcopy(nums))

    assert it.rev().enumerate().to_list() == [(0, 3), (1, 2), (2, 1)]


def test_filter() -> None:
    nums = [1, 2, 3, 4, 5]
    it = iterator_from(deepcopy(nums))

    f = lambda x: x % 2 == 0

    lib_filter = it.filter(f).to_list()
    native_filter = list(filter(f, nums))

    assert lib_filter == native_filter


def test_filter_consecutive() -> None:
    nums = [2, 4, 6, 1, 2, 3]
    it = iterator_from(deepcopy(nums))

    f = lambda x: x % 2 == 0

    lib_filter = it.filter(f).to_list()
    native_filter = list(filter(f, nums))

    assert lib_filter == native_filter


def test_filter_negative_twice() -> None:
    nums = [1, 2, 3, 4, 5]
    it = iterator_from(deepcopy(nums))

    f = lambda x: x < 0

    lib_filter = it.filter(f).filter(f).to_list()
    native_filter = list(filter(f, filter(f, nums)))

    assert lib_filter == native_filter


def test_take() -> None:
    nums = [1, 2, 3, 4, 5]
    lib_it = iterator_from(deepcopy(nums))
    native_it = iter(deepcopy(nums))

    lib_taken = lib_it.take(1).to_list()
    native_taken = list(islice(native_it, 1))

    assert lib_taken == native_taken
    assert lib_it.to_list() == list(native_it)


def test_take_2() -> None:
    nums = [1, 2, 3, 4, 5]
    lib_it = iterator_from(deepcopy(nums))
    native_it = iter(deepcopy(nums))

    lib_taken = lib_it.take(2).to_list()
    native_taken = list(islice(native_it, 2))

    assert lib_taken == native_taken
    assert lib_it.to_list() == list(native_it)


def test_map_take() -> None:
    nums = [1, 2, 3, 4, 5]
    lib_it = iterator_from(deepcopy(nums))
    native_it = iter(deepcopy(nums))

    f = lambda x: x + 10

    lib_res = lib_it.map(f).take(3).to_list()
    native_res = list(islice(map(f, native_it), 3))

    assert lib_res == native_res
