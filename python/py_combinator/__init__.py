from py_combinator import _py_combinator as rs
from py_combinator._py_combinator import AnyIterator

from .wrappers import no_use_import, use_import

__doc__ = rs.__doc__

__all__ = []

if hasattr(rs, "__all__"):
    __all__ += rs.__all__

__all__ += [
    "AnyIterator",
    "no_use_import",
    "use_import",
]
