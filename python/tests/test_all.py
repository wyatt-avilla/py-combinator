import pytest
import py_combinator


def test_sum_as_string():
    assert py_combinator.sum_as_string(1, 1) == "2"
