from py_combinator import _py_combinator as rs


def no_use_import() -> None:
    print("no import")


def use_import() -> None:
    print("used import")
    print(rs.sum_as_string(30, 40))
