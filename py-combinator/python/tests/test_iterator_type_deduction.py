from py_combinator import (
    BaseIterator,
    iterator_from,
)

# ruff: noqa: E731 S101


class TestIteratorTypeDeduction:
    def test_list_iter_type_deduction(self) -> None:
        nums = [1, 2, 3, 4, 5]
        lib_it = iterator_from(iter(nums))
        assert isinstance(lib_it, BaseIterator)

    def test_tuple_iter_type_deduction(self) -> None:
        nums = (1, 2, 3, 4, 5)
        lib_it = iterator_from(iter(nums))
        assert isinstance(lib_it, BaseIterator)
