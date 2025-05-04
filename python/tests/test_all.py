import pytest
import my_project


def test_sum_as_string():
    assert my_project.sum_as_string(1, 1) == "2"
