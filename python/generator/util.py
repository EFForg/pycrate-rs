from generator.namer import Name


def indent(s: str, num_indents=1) -> str:
    indentation = ' ' * 4 * num_indents
    lines = [indentation + line for line in s.split('\n')]
    return '\n'.join(lines)


def upper_camel_case(s: str) -> str:
    return Name(s).cc()


def snake_case(s: str) -> str:
    return Name(s).sc()
