from generator.namer import Name


def indent(s: str, num_indents=1) -> str:
    """Indents each line of the input string"""
    indentation = ' ' * 4 * num_indents
    lines = [indentation + line for line in s.split('\n')]
    return '\n'.join(lines)


def upper_camel_case(s: str) -> str:
    """Returns an upper-camel-cased version of the input, e.g.:
        upper_camel_case("foo bar (baz)") -> "FooBarBaz"
    """
    return Name(s).cc()


def snake_case(s: str) -> str:
    """Returns a lower-snake-cased version of the input, e.g.:
        snake_case("FooBar (baz)") -> "foo_bar_baz
    """
    return Name(s).sc()
