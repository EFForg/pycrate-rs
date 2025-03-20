from typing import List, Optional
from pycrate_core import elt
from pycrate_core.base import Uint, Buf, Uint8, Uint16
from enum import StrEnum, IntEnum, auto

from generator.util import indent, upper_camel_case, snake_case


class Layer3Type(StrEnum):
    Type1V = 'Type1V'
    Type1TV = 'Type1TV'
    Type2 = 'Type2'
    Type3V = 'Type3V'
    Type3TV = 'Type3TV'
    Type4LV = 'Type4LV'
    Type4TLV = 'Type4TLV'
    Type6LVE = 'Type6LVE'
    Type6TLVE = 'Type6TLVE'

    def is_sized(self) -> bool:
        return self in [
            Layer3Type.Type1V,
            Layer3Type.Type1TV,
            Layer3Type.Type4LV,
            Layer3Type.Type4TLV,
            Layer3Type.Type6LVE,
            Layer3Type.Type6TLVE,
        ]

    def is_variable_length(self) -> bool:
        return self in [
            Layer3Type.Type4LV,
            Layer3Type.Type4TLV,
            Layer3Type.Type6LVE,
            Layer3Type.Type6TLVE,
        ]

    def is_tagged(self) -> bool:
        return self in [
            Layer3Type.Type1TV,
            Layer3Type.Type3TV,
            Layer3Type.Type4TLV,
            Layer3Type.Type6TLVE,
        ]


class Layer3Wrapper:
    def __init__(self, obj: elt.Envelope) -> None:
        self.type = Layer3Type(type(obj).__name__)
        if self.type.is_tagged():
            assert obj[0]._name == 'T'
            assert isinstance(obj[0], Uint)
            self.tag = obj[0].get_val()
        else:
            self.tag = None

    def get_inner_idx(self) -> int:
        return {
            Layer3Type.Type1V: 0,
            Layer3Type.Type1TV: 1,
            Layer3Type.Type3V: 0,
            Layer3Type.Type3TV: 1,
            Layer3Type.Type4LV: 1,
            Layer3Type.Type4TLV: 2,
            Layer3Type.Type6LVE: 1,
            Layer3Type.Type6TLVE: 2,
        }[self.type]


def get_layer3_wrapper(obj: elt.Envelope) -> Optional[Layer3Wrapper]:
    try:
        return Layer3Wrapper(obj)
    except ValueError:
        return None


def derives(partial_eq=False) -> str:
    traits = [
        'DekuRead',
        # 'DekuWrite', # TODO: implement DekuWrite for Layer3 types
        'Debug',
        'Serialize',
        'Clone',
    ]
    if partial_eq:
        traits.append('PartialEq')
    return f'#[derive({', '.join(traits)})]'


class RustPrimitiveType(IntEnum):
    U8 = auto()
    I8 = auto()
    U16 = auto()
    I16 = auto()
    U32 = auto()
    I32 = auto()
    VecU8 = auto()

    def rust_type_name(self) -> str:
        if self == RustPrimitiveType.VecU8:
            return 'Vec<u8>'
        return self.name.lower()

    def is_big_endian(self) -> bool:
        return self in [
            RustPrimitiveType.U16,
            RustPrimitiveType.I16,
            RustPrimitiveType.U32,
            RustPrimitiveType.I32,
        ]

    @staticmethod
    def from_pycrate(obj: elt.Atom) -> 'RustPrimitiveType':
        if isinstance(obj, Buf):
            return RustPrimitiveType.VecU8
        if isinstance(obj, Uint8):
            return RustPrimitiveType.U8
        if isinstance(obj, Uint16):
            return RustPrimitiveType.U16
        type_name = type(obj).__name__
        bit_len = obj.get_bl()
        assert type_name == 'Uint'
        if bit_len <= 8:
            return RustPrimitiveType.U8
        elif bit_len <= 16:
            return RustPrimitiveType.U16
        elif bit_len <= 32:
            return RustPrimitiveType.U32
        raise ValueError('unknown primtive type', obj)


class RustStructField:
    def __init__(
        self,
        name: str,
        type: 'Optional[RustPrimitiveType | RustStruct | RustEnum]',
        layer3_wrapper: Optional[Layer3Wrapper],
        bit_length: Optional[int],
        bit_padding: Optional[int],
    ):
        self.type = type
        self.name = snake_case(name)
        self.layer3_wrapper = layer3_wrapper
        self.bit_length = bit_length
        self.bit_padding = bit_padding

        # whether this field should be conditionally parsed based on how many
        # bytes remain
        self.is_optional = False

        # some structs have a `Vec<u8>` that should consume the rest of the
        # input bytes
        self.is_final_buf = False

    def _deku_attrs(self) -> str:
        deku_attrs = []
        ctx = []
        if self.layer3_wrapper is not None:
            if self.layer3_wrapper.tag is not None:
                ctx.append(f'Tag({self.layer3_wrapper.tag})')
            if self.bit_length is not None:
                if self.bit_length % 8 == 0:
                    deku_attrs.append(f'bytes = {int(self.bit_length / 8)}')
                else:
                    deku_attrs.append(f'bits = {self.bit_length}')
        else:
            if self.type == RustPrimitiveType.VecU8:
                if self.is_final_buf:
                    deku_attrs.append('count = "byte_size - deku::byte_offset"')
                else:
                    assert self.bit_length is not None
                    assert self.bit_length % 8 == 0
                    byte_length = int(self.bit_length / 8)
                    deku_attrs.append(f'count = "{byte_length}"')
            elif self.bit_length is not None and not isinstance(self.type, RustEnum):
                if self.bit_length % 8 == 0:
                    deku_attrs.append(f'bytes = {int(self.bit_length / 8)}')
                else:
                    deku_attrs.append(f'bits = {self.bit_length}')

        if isinstance(self.type, RustStruct):
            if self.type.is_variable_bitfield or self.type.contains_final_buf():
                ctx.append("NeedsByteSize")
        elif self._is_layer3_buffer():
            ctx.append("NeedsByteSize")

        if self.bit_padding is not None:
            deku_attrs.append(f'pad_bits_before = "{self.bit_padding}"')

        if self.type is not None and not isinstance(self.type, RustEnum) and self.type.is_big_endian():
            deku_attrs.append('endian = "big"')

        if self.is_optional:
            deku_attrs.append('cond = "deku::byte_offset < byte_size"')
            if isinstance(self.type, RustEnum):
                default_name = f"{self.type.name}::{self.type.variants[0].name}"
                deku_attrs.append(f'default = "{default_name}"')

        if len(ctx):
            deku_attrs.append(f'ctx = "{', '.join(ctx)}"')
        deku_part = ''
        if len(deku_attrs):
            deku_part = f'#[deku({', '.join(deku_attrs)})] '
        return deku_part

    def _is_layer3_buffer(self) -> bool:
        is_wrapped = self.layer3_wrapper is not None
        is_buf = self.type == RustPrimitiveType.VecU8
        return is_wrapped and is_buf

    def to_rust(self) -> str:
        # special case for Type4TLV<Vec<u8>>
        type_name = '()' if self.type is None else self.type.rust_type_name()
        if self.layer3_wrapper is not None:
            if self._is_layer3_buffer():
                type_name = 'Layer3Buffer'
            wrapper_name = str(self.layer3_wrapper.type)
            type_name = f"{wrapper_name}<{type_name}>"
        deku_part = self._deku_attrs()
        return f'{deku_part}pub {self.name}: {type_name},'


class RustStruct:
    def __init__(
        self,
        name: str,
    ) -> None:
        self.fields: list[RustStructField] = []
        self.pyobj_indices: list[Optional[int]] = []
        self.name = upper_camel_case(name)

        # Some of these structs are variable-sized bitfields which whose fields
        # should be optionally parsed based on the byte-size of the entire
        # struct
        self.is_variable_bitfield = any([
            self.name.endswith('Cap'),
            self.name == 'EPSNetFeat',
            self.name == 'APNAMBR',
        ])

    @staticmethod
    def from_pycrate(
        obj: elt.Envelope,
    ) -> 'RustStruct':
        return RustStruct(obj._name)

    def add_field(self, field: RustStructField, pyobj_index: Optional[int]) -> None:
        if self.is_variable_bitfield:
            field.is_optional = True
        self.fields.append(field)
        self.pyobj_indices.append(pyobj_index)

    def _fix_all_duplicates(self) -> None:
        dupe = self._find_duplicate_field_name()
        while dupe is not None:
            self._fix_duplicate_field(dupe)
            dupe = self._find_duplicate_field_name()

    def _find_duplicate_field_name(self) -> Optional[str]:
        names = [field.name for field in self.fields]
        for name in names:
            if names.count(name) > 1:
                return name
        return None

    def _fix_duplicate_field(self, name: str) -> None:
        dupes = [field for field in self.fields if field.name == name]
        for i, dupe in enumerate(dupes):
            dupe.name += f"_{i + 1}"

    def is_big_endian(self) -> bool:
        return False

    def contains_final_buf(self) -> bool:
        if len(self.fields):
            return self.fields[-1].is_final_buf
        else:
            return False

    def to_rust(self) -> str:
        self._fix_all_duplicates()
        deku_ctx = ''
        if self.is_variable_bitfield or self.contains_final_buf():
            deku_ctx = '\n#[deku(ctx = "ByteSize(byte_size): ByteSize")]'
        return f'''\
{derives()}{deku_ctx}
pub struct {self.name} {{
{self._fields_to_rust()}
}}'''

    def _fields_to_rust(self) -> str:
        fields = [field.to_rust() for field in self.fields]
        return '\n'.join([indent(field) for field in fields])

    def rust_type_name(self) -> str:
        return self.name


class RustEnumVariant:
    def __init__(self, name: str, value: int):
        self.name = upper_camel_case(name)
        self.values = [value]

    def to_rust(self) -> str:
        id_pat = ' | '.join([str(v) for v in self.values])
        return f'#[deku(id_pat = "{id_pat}")] {self.name},'


class RustEnum:
    def __init__(
        self,
        name: str,
        type: RustPrimitiveType,
        bit_length: int,
    ) -> None:
        self.variants: List[RustEnumVariant] = []
        self.type = type
        self.bit_length = bit_length
        self.name = upper_camel_case(name)

    def is_big_endian(self) -> bool:
        return self.type.is_big_endian()

    @staticmethod
    def from_pycrate(obj: elt.Atom, prefix: str) -> 'RustEnum':
        rust_enum = RustEnum(
            prefix + obj._name,
            RustPrimitiveType.from_pycrate(obj),
            obj.get_bl(),
        )
        assert obj._dic is not None
        for val, name in obj._dic.items():
            rust_enum.add_variant(RustEnumVariant(name, val))
        return rust_enum

    def add_variant(self, variant: RustEnumVariant):
        for existing in self.variants:
            if variant.name == existing.name:
                existing.values += variant.values
                return
        self.variants.append(variant)

    def rust_type_name(self) -> str:
        return self.name

    def to_rust(self) -> str:
        deku_attrs = [
            f'id_type = "{self.type.rust_type_name()}"',
            f'bits = {self.bit_length}',
        ]
        if self.is_big_endian():
            deku_attrs.append('endian = "big"')
        # FIXME: we can't support a catchall variant which stores the value
        # until https://github.com/sharksforarms/deku/issues/533 is fixed
        # other_type_part = f'#[deku(bits = {self.bit_length})] {self.type.rust_type_name()}'
        # other_variant = f'#[deku(id_pat = "_")] Other({other_type_part}),'
        other_variant = '#[deku(id_pat = "_")] Other,'
        return f'''\
{derives(partial_eq=True)}
#[deku({', '.join(deku_attrs)})]
pub enum {self.name} {{
{self._variants_to_rust()}
{indent(other_variant)}
}}'''

    def _variants_to_rust(self) -> str:
        variants = [var.to_rust() for var in self.variants]
        return '\n'.join([indent(var) for var in variants])
