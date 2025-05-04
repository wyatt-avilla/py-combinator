from .py_combinator import sum_as_string


def no_use_import() -> None:
    print("no import")


def use_import() -> None:
    print("used import")
    print(sum_as_string(30, 40))
