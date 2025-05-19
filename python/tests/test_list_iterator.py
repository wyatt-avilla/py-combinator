from copy import deepcopy

from py_combinator import ListIterator

# ruff: noqa: E731 S101


def test_map_addition() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))
    f = lambda x: x + 1

    lib_mapped = it.map(f).to_list()
    native_mapped = list(map(f, nums))

    assert lib_mapped == native_mapped


def test_multiple_maps() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))
    f1 = lambda x: x + 1
    f2 = lambda x: x * 2

    lib_mapped = it.map(f1).map(f2).to_list()
    native_mapped = list(map(f2, map(f1, nums)))

    assert lib_mapped == native_mapped


def test_list_conversion() -> None:
    nums = [1, 2, 3]

    it = ListIterator(nums)

    assert it.to_list() == nums


def test_chaining() -> None:
    c1 = ListIterator([1, 2, 3])
    c2 = c1.map(lambda x: x)
    c3 = c2.map(lambda x: x)

    assert c1 is c2 is c3


def test_uncalled_count() -> None:
    c1 = ListIterator([1, 2, 3])

    assert c1.uncalled == 0

    c1.map(lambda x: x)

    assert c1.uncalled == 1

    c1.to_list()

    assert c1.uncalled == 0


def test_fold() -> None:
    nums = [1, 2, 3]
    it = ListIterator(nums)

    f = lambda acc, x: acc * x
    acc = 1

    libfolded = it.fold(acc, f)
    [acc := f(acc, x) for x in nums]

    assert acc == libfolded


def test_map_fold() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))

    f_add = lambda x: x + 1
    f_fold = lambda acc, x: acc * x
    acc = 1

    libfolded = it.map(f_add).fold(acc, f_fold)
    [acc := f_fold(acc, x) for x in map(f_add, nums)]

    assert acc == libfolded


def test_reverse() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))

    lib_reversed = it.rev().to_list()
    native_reversed = list(reversed(nums))

    assert native_reversed == lib_reversed


def test_map_reverse() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))

    f = lambda x: x + 1

    lib_reversed = it.map(f).rev().to_list()
    native_reversed = list(reversed(list(map(f, nums))))

    assert native_reversed == lib_reversed


def test_enumerate() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))

    lib_enumerate = it.enumerate().to_list()
    native_enumerate = list(enumerate(nums))

    assert lib_enumerate == native_enumerate


def test_pass_by_ref_semantics() -> None:
    nums = [1, 2, 3]
    it = ListIterator(nums)

    assert it.to_list() is nums


def test_enumerate_rev() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))

    assert it.enumerate().rev().to_list() == [(2, 3), (1, 2), (0, 1)]


def test_rev_enumerate() -> None:
    nums = [1, 2, 3]
    it = ListIterator(deepcopy(nums))

    assert it.rev().enumerate().to_list() == [(0, 3), (1, 2), (2, 1)]


def test_filter() -> None:
    nums = [1, 2, 3, 4, 5]
    it = ListIterator(deepcopy(nums))

    f = lambda x: x % 2 == 0

    lib_filter = it.filter(f).to_list()
    native_filter = list(filter(f, nums))

    assert lib_filter == native_filter


def test_filter_consecutive() -> None:
    nums = [2, 4, 6, 1, 2, 3]
    it = ListIterator(deepcopy(nums))

    f = lambda x: x % 2 == 0

    lib_filter = it.filter(f).to_list()
    native_filter = list(filter(f, nums))

    assert lib_filter == native_filter


def test_filter_negative_twice() -> None:
    nums = [1, 2, 3, 4, 5]
    it = ListIterator(deepcopy(nums))

    f = lambda x: x < 0

    lib_filter = it.filter(f).filter(f).to_list()
    native_filter = list(filter(f, filter(f, nums)))

    assert lib_filter == native_filter
