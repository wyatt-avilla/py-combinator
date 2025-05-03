import py_combinator


def test_func() -> None:
    print("hello test func")
    print(py_combinator.sum_as_string(10, 20))


if __name__ == "__main__":
    print("hi main")
    test_func()
