from py_combinator import _py_combinator as rs
from py_combinator._py_combinator import (
    PyBaseIterator as BaseIterator,
)
from py_combinator._py_combinator import (
    PyDoubleEndedIterator as DoubleEndedIterator,
)
from py_combinator._py_combinator import (
    PyExactSizeIterator as ExactSizeIterator,
)
from py_combinator._py_combinator import (
    PySizedDoubleEndedIterator as SizedDoubleEndedIterator,
)
from py_combinator._py_combinator import iterator_from

__doc__ = rs.__doc__

__all__ = []

if hasattr(rs, "__all__"):
    __all__ += rs.__all__

__all__ += [
    "BaseIterator",
    "DoubleEndedIterator",
    "ExactSizeIterator",
    "SizedDoubleEndedIterator",
    "iterator_from",
]
