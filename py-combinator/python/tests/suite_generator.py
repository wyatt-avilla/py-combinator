#!/usr/bin/env python3

import argparse
import inspect
import itertools
import sys
from collections import defaultdict
from collections.abc import Callable, Iterable
from copy import deepcopy
from dataclasses import dataclass
from typing import Any, ClassVar, Union

from py_combinator import SizedDoubleEndedIterator, iterator_from

SUPPORTED_WRAPPERS = (SizedDoubleEndedIterator,)

# enable lint when more wrappers are supported
SupportedWrappers = Union[SizedDoubleEndedIterator]  # noqa: UP007

FunctionT = Union["NumericLambda"]


@dataclass
class InbuiltFunction:
    no_args_type = Callable[
        [Iterable[Any]],
        Any,
    ]

    args_type = Callable[
        [FunctionT, Iterable[Any]],
        Any,
    ]

    name: str
    fn: args_type | no_args_type


@dataclass
class LibraryMethod:
    no_args_type = Callable[
        [SupportedWrappers],
        SupportedWrappers,
    ]

    args_type = Callable[
        [FunctionT, SupportedWrappers],
        SupportedWrappers,
    ]

    name: str
    fn: args_type | no_args_type


@dataclass
class NumericLambda:
    name: str
    fn: Callable[[int | float], int | float]


@dataclass
class NumericPredicate:
    name: str
    fn: Callable[[int | float], int | float]


@dataclass
class TestCaseResult:
    pass


@dataclass
class Pass(TestCaseResult):
    def __str__(self) -> str:
        return "PASS"


@dataclass
class Fail(TestCaseResult):
    expected: Any
    got: Any
    error: Exception | None = None

    def __str__(self) -> str:
        return "FAIL"


@dataclass
class Skip(TestCaseResult):
    error: TypeError

    def __str__(self) -> str:
        return "SKIP"


class TestCase:
    all_cases: ClassVar = defaultdict(lambda: 0)

    def __init__(
        self,
        initial_data: Iterable[Any],
        chain: list[
            tuple[InbuiltFunction, LibraryMethod]
            | tuple[InbuiltFunction, LibraryMethod, NumericLambda]
        ],
    ) -> None:
        self.initial_data: Iterable[Any] = initial_data
        self.iter = iterator_from(initial_data)
        self.chain: list[
            tuple[InbuiltFunction, LibraryMethod]
            | tuple[InbuiltFunction, LibraryMethod, NumericLambda]
        ] = chain

    @property
    def name(self) -> str:
        name = f"{self.wrapper.__name__}_" + "_".join(
            tup[1].name + (f"_{tup[2].name}" if len(tup) == 3 else "")  # noqa: PLR2004
            for tup in self.chain
        )

        potential_id = self.all_cases[name]
        if potential_id == 0:
            self.all_cases[name] += 1
            return name

        return f"{name}_{potential_id}"

    def run(self) -> TestCaseResult:
        lib = self.iter
        native = deepcopy(self.initial_data)

        try:
            for native_fn, lib_fn, *rest in self.chain:
                lam = rest[0] if rest else None

                if lam is None:
                    lib = lib_fn.fn(lib)
                    native = native_fn.fn(native)
                else:
                    lib = lib_fn.fn(lam.fn, lib)
                    native = native_fn.fn(lam.fn, native)
            lib = lib.to_list()
            native = list(native)
        except TypeError as e:
            return Skip(e)
        except Exception as e:  # noqa: BLE001
            return Fail(native, lib, e)

        return Pass() if lib == native else Fail(native, lib)


FUNCTION_METHOD_PAIRS: list[tuple[InbuiltFunction, LibraryMethod]] = [
    (
        InbuiltFunction("reversed", lambda it: list(reversed(it))),
        LibraryMethod("rev", lambda w: w.rev()),
    ),
    (
        InbuiltFunction("map", lambda fn, it: list(map(fn, it))),
        LibraryMethod("map", lambda fn, w: w.map(fn)),
    ),
    (
        InbuiltFunction("enumerate", lambda it: list(enumerate(it))),
        LibraryMethod("enumerate", lambda w: w.enumerate()),
    ),
    (
        InbuiltFunction("filter", lambda fn, it: list(filter(fn, it))),
        LibraryMethod("filter", lambda fn, w: w.filter(fn)),
    ),
]

NUMERIC_FUNCS = [
    NumericLambda("add_one", lambda x: x + 1),
    NumericLambda("double", lambda x: x * 2),
    NumericLambda("negate", lambda x: -x),
    NumericLambda("square", lambda x: x**2),
    NumericLambda("abs", lambda x: abs(x)),
]

NUMERIC_PREDICATES = [
    NumericPredicate("is_even", lambda x: x % 2 == 0),
    NumericPredicate("is_odd", lambda x: x % 2 == 1),
    NumericPredicate("is_positive", lambda x: x > 0),
    NumericPredicate("is_negative", lambda x: x < 0),
]

INT_TEST_DATA = [
    [1, 2, 3, 4, 5],
    [],
    [10] * 5,
    list(range(10)),
]


def generate_matrix(depth: int) -> list[TestCase]:
    matrix: list[TestCase] = []

    for data in INT_TEST_DATA:
        for d in range(1, depth + 1):
            for combo in itertools.product(FUNCTION_METHOD_PAIRS, repeat=d):
                enriched_combos = [[]]
                for fn_pair in combo:
                    inbuilt, method = fn_pair

                    sig = inspect.signature(inbuilt.fn)
                    params = sig.parameters
                    expects_numeric = len(params) == 2  # noqa: PLR2004

                    if expects_numeric:
                        enriched_combos = [
                            [*existing, (inbuilt, method, numeric)]
                            for existing in enriched_combos
                            for numeric in NUMERIC_FUNCS + NUMERIC_PREDICATES
                        ]
                    else:
                        enriched_combos = [
                            [*existing, fn_pair] for existing in enriched_combos
                        ]

                matrix.extend(TestCase(data, enriched) for enriched in enriched_combos)

    return matrix


def green(s: str) -> str:
    return f"\033[32m{s}\033[0m"


def red(s: str) -> str:
    return f"\033[31m{s}\033[0m"


def yellow(s: str) -> str:
    return f"\033[33m{s}\033[0m"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run test matrix.")
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose output",
    )

    parser.add_argument(
        "-vv",
        "--very_verbose",
        action="store_true",
        help="Enable verbose output",
    )

    parser.add_argument(
        "-d",
        "--depth",
        type=int,
        default=3,
        help="Set maximum depth",
    )

    args = parser.parse_args()
    verbose = args.verbose
    very_verbose = args.very_verbose
    depth = args.depth

    passed_count = 0
    failed_count = 0
    skipped_count = 0

    for test_case in generate_matrix(depth=depth):
        result = test_case.run()

        if isinstance(result, Pass):
            passed_count += 1
            if verbose or very_verbose:
                print(f"{green(str(result))} {test_case.name}")
        elif isinstance(result, Fail):
            failed_count += 1
            if verbose or very_verbose:
                print(f"{red(str(result))} {test_case.name}")
            if very_verbose:
                if result.error is not None:
                    print(f"Error: \n{result.error}")

                print(f"From: \n{test_case.initial_data}")
                print(f"Expected: \n{result.expected}")
                print(f"Got: \n{result.got}")
        elif isinstance(result, Skip):
            skipped_count += 1
            if verbose:
                print(f"{yellow(str(result))} {test_case.name}")
            if very_verbose:
                print(f"{yellow(str(result))} {test_case.name}  ({result.error})")

    if verbose or very_verbose:
        print("=" * 80)

    print(f"TOTAL: {sum((passed_count, failed_count, skipped_count))}")
    print(
        f"({green(str(passed_count))} passed, {red(str(failed_count))} failed, {yellow(str(skipped_count))} skipped)",
    )

    if failed_count > 0:
        sys.exit(1)
