import unittest


def attrfy(attrs: list[str]) -> str:
    if len(attrs) == 0:
        return ''
    return f'#[deku({', '.join(attrs)})] '


class DekuAttributes:
    def __init__(self) -> None:
        self.is_buf = False
        self.is_final_buf = False
        self.default_enum_variant = None
        self.bit_padding = None
        self.is_big_endian = False
        self.is_optional = False
        self.needs_byte_size = False
        self.is_wrapped = False
        self.size = None
        self.tag = None

    def set_tag(self, tag: int) -> None:
        self.tag = tag

    def set_big_endian(self, is_big_endian: bool) -> None:
        self.is_big_endian = is_big_endian

    def set_size(self, bits: int) -> None:
        if bits % 8 == 0:
            self.size = ('bytes', int(bits / 8))
        else:
            self.size = ('bits', bits)

    def mark_as_buf(self, final_buf=False) -> None:
        self.is_buf = True
        self.is_final_buf = final_buf

    def set_is_optional(self, is_optional: bool) -> None:
        self.is_optional = is_optional

    def set_needs_byte_size(self, needs_byte_size: bool) -> None:
        self.needs_byte_size = needs_byte_size

    def _is_enum(self) -> bool:
        return self.default_enum_variant is not None

    def mark_as_enum(self, enum_name: str, default_variant_name: str) -> None:
        self.default_enum_variant = (enum_name, default_variant_name)

    def mark_as_wrapped(self) -> None:
        self.is_wrapped = True

    def set_bit_padding(self, bit_padding: int) -> None:
        self.bit_padding = bit_padding

    def _build_ctx(self, attrs: list[str]) -> None:
        ctx = []
        if self.tag is not None:
            ctx.append(f'Tag({self.tag})')
        if self.needs_byte_size:
            ctx.append("NeedsByteSize")
        if len(ctx):
            attrs.append(f'ctx = "{', '.join(ctx)}"')

    def _set_size_or_count(self, attrs: list[str]) -> None:
        if self.is_wrapped or not self._is_enum():
            if self.size:
                units, value = self.size
                if not self.is_wrapped and self.is_final_buf:
                    attrs.append('count = "byte_size - deku::byte_offset"')
                elif not self.is_wrapped and self.is_buf:
                    if units != 'bytes':
                        raise ValueError(f'buf type had {value} bits')
                    attrs.append(f'count = "{value}"')
                else:
                    attrs.append(f'{units} = {value}')

    def to_rust(self) -> str:
        attrs = []
        self._set_size_or_count(attrs)
        if self.is_wrapped or not self._is_enum():
            if self.bit_padding:
                attrs.append(f'pad_bits_before = "{self.bit_padding}"')
        if not self._is_enum() and self.is_big_endian:
            attrs.append('endian = "big"')
        if self.is_optional:
            attrs.append('cond = "deku::byte_offset < byte_size"')
            if self.default_enum_variant is not None:
                enum_name, variant_name = self.default_enum_variant
                attrs.append(f'default = "{enum_name}::{variant_name}"')
        self._build_ctx(attrs)
        return attrfy(attrs)


class TestNamer(unittest.TestCase):
    def test_empty(self):
        attr = DekuAttributes()
        assert attr.to_rust() == ''

    def test_tag(self):
        attr = DekuAttributes()
        attr.set_tag(10)
        assert attr.to_rust() == attrfy(['ctx = "Tag(10)"'])

        attr.set_size(10)
        assert attr.to_rust() == attrfy(['bits = 10', 'ctx = "Tag(10)"'])

    def test_needs_byte_size(self):
        attr = DekuAttributes()
        attr.set_needs_byte_size(True)
        assert attr.to_rust() == attrfy(['ctx = "NeedsByteSize"'])

        # make sure Tag always appears before NeedsByteSize
        attr.set_tag(10)
        assert attr.to_rust() == attrfy(['ctx = "Tag(10), NeedsByteSize"'])

    def test_is_optional(self):
        attr = DekuAttributes()
        attr.set_is_optional(True)
        assert attr.to_rust() == attrfy(['cond = "deku::byte_offset < byte_size"'])

        attr.mark_as_enum('EnumName', 'DefaultVariant')
        assert attr.to_rust() == attrfy([
            'cond = "deku::byte_offset < byte_size"',
            'default = "EnumName::DefaultVariant"'
        ])

    def test_bit_padding(self):
        attr = DekuAttributes()
        attr.set_bit_padding(4)
        assert attr.to_rust() == attrfy(['pad_bits_before = "4"'])

        attr.mark_as_enum('EnumName', 'DefaultVariant')
        assert attr.to_rust() == ''

    def test_endian(self):
        attr = DekuAttributes()
        attr.set_big_endian(True)
        assert attr.to_rust() == attrfy(['endian = "big"'])

        attr.mark_as_enum('EnumName', 'DefaultVariant')
        assert attr.to_rust() == ''

        attr.mark_as_wrapped()
        assert attr.to_rust() == ''

    def test_size(self):
        attr = DekuAttributes()
        attr.set_size(0)
        assert attr.to_rust() == attrfy(['bytes = 0'])
        attr.set_size(8)
        assert attr.to_rust() == attrfy(['bytes = 1'])
        attr.set_size(7)
        assert attr.to_rust() == attrfy(['bits = 7'])

    def test_enum(self):
        attr = DekuAttributes()
        attr.mark_as_enum('EnumName', 'DefaultVariant')
        attr.set_size(0)
        assert attr.to_rust() == ''
        attr.set_size(8)
        assert attr.to_rust() == ''

        attr.mark_as_wrapped()
        assert attr.to_rust() == attrfy(['bytes = 1'])

    def test_is_wrapped(self):
        attr = DekuAttributes()
        attr.mark_as_buf()
        attr.mark_as_wrapped()
        attr.set_size(8)
        assert attr.to_rust() == attrfy(['bytes = 1'])

    def test_buf(self):
        attr = DekuAttributes()
        attr.mark_as_buf()
        attr.set_size(0)
        assert attr.to_rust() == attrfy(['count = "0"'])
        attr.set_size(8)
        assert attr.to_rust() == attrfy(['count = "1"'])
        attr.set_size(7)
        self.assertRaises(ValueError, attr.to_rust)

        attr.mark_as_buf(True)
        attr.set_size(0)
        assert attr.to_rust() == attrfy([
            'count = "byte_size - deku::byte_offset"'
        ])
        attr.set_size(8)
        assert attr.to_rust() == attrfy([
            'count = "byte_size - deku::byte_offset"'
        ])
