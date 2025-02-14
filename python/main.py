from typing import Dict, List, Optional, Tuple, Union, Any
from pycrate_core import elt
from pycrate_core.base import Uint, Buf, Uint8, Uint16
from pycrate_csn1.csnobj import CSN1List
from pycrate_mobile import NASLTE
from pycrate_mobile.TS24007 import IE, Layer3E
from pycrate_mobile.TS24301_EMM import EMMHeader
from enum import StrEnum, IntEnum, auto

from pycrate_mobile.TS24301_IE import LCSClientId


RESERVED_WORDS = [
    'type',
]


class Layer3Wrapper(StrEnum):
    Type1V = 'Type1V'
    Type1TV = 'Type1TV'
    Type2 = 'Type2'
    Type3V = 'Type3V'
    Type3TV = 'Type3TV'
    Type4LV = 'Type4LV'
    Type4TLV = 'Type4TLV'
    Type6LVE = 'Type6LVE'
    Type6TLVE = 'Type6TLVE'


def get_layer3_wrapper(obj: elt.Envelope) -> Optional[Layer3Wrapper]:
    try:
        return Layer3Wrapper(type(obj).__name__)
    except ValueError:
        return None


def upper_camel_case(s: str) -> str:
    s = sanitize(s)
    if ' ' in s:
        return s.title().replace(' ', '')
    else:
        return s


def snake_case(s: str) -> str:
    s = sanitize(s)
    snake_cased = s.replace(' ', '_').lower()
    if snake_cased in RESERVED_WORDS:
        snake_cased = snake_cased[:-1]
    return snake_cased


def sanitize(s: str) -> str:
    s = replace_forbidden_character(s)
    return s


def replace_forbidden_character(s: str) -> str:
    s = s.replace('+', 'Plus')
    s = s.replace('-', 'Minus')
    s = s.replace('(', '')
    s = s.replace(')', '')
    return s


def derives() -> str:
    traits = [
        'DekuRead',
        'DekuWrite',
        'Debug',
        'Clone'
    ]
    return f'#[derive({', '.join(traits)})]'


class RustPrimitiveType(IntEnum):
    U8 = auto()
    I8 = auto()
    U16 = auto()
    I16 = auto()
    U32 = auto()
    I32 = auto()
    F32 = auto()
    Char = auto()
    VecU8 = auto()

    def rust_type_name(self) -> str:
        if self == RustPrimitiveType.VecU8:
            return 'Vec<u8>'
        return self.name.lower()

    @staticmethod
    def from_pycrate(obj: elt.Atom) -> 'RustPrimitiveType':
        if isinstance(obj, Buf):
            return RustPrimitiveType.VecU8
        if isinstance(obj, Uint8):
            return RustPrimitiveType.U8
        if isinstance(obj, Uint16):
            return RustPrimitiveType.U16
        type_name = type(obj).__name__
        return {
            'Uint': RustPrimitiveType.U32,
        }[type_name]


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

    def to_rust(self) -> str:
        type_name = 'None' if self.type is None else self.type.rust_type_name()
        if self.layer3_wrapper is not None:
            wrapper_name = str(self.layer3_wrapper)
            type_name = f"{wrapper_name}<{type_name}>"
        deku_attrs = []
        # deku bits attribute goes on the enum decl
        if self.bit_length is not None and not isinstance(self.type, RustEnum):
            deku_attrs.append(f'bits = {self.bit_length}')
        if self.bit_padding is not None:
            deku_attrs.append(f'pad_bits_before = {self.bit_padding}')
        deku_part = ''
        if len(deku_attrs):
            deku_part = f'#[deku({', '.join(deku_attrs)})] '
        return f'{deku_part}{self.name}: {type_name},'


class RustStruct:
    def __init__(
        self,
        name: str,
        layer3_wrapper: Optional[Layer3Wrapper] = None,
    ) -> None:
        self.fields: List[RustStructField] = []
        self.layer3_wrapper = layer3_wrapper
        self.name = upper_camel_case(name)

    @staticmethod
    def from_pycrate(
        obj: elt.Envelope,
    ) -> 'RustStruct':
        return RustStruct(obj._name, get_layer3_wrapper(obj))

    def add_field(self, field: RustStructField) -> None:
        self.fields.append(field)

    def to_rust(self) -> str:
        return f'''\
{derives()}
struct {self.name} {{
{self._fields_to_rust()}
}}'''

    def _fields_to_rust(self) -> str:
        fields = [field.to_rust() for field in self.fields]
        return '\n'.join([indent(field) for field in fields])

    def rust_type_name(self) -> str:
        return self.name


def indent(s: str, num_indents=1) -> str:
    indentation = ' ' * 4 * num_indents
    return indentation + s


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
        self.layer3_wrapper: Optional[Layer3Wrapper] = None
        self.name = upper_camel_case(name)

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
        return f'''\
{derives()}
#[deku(id_type = "{self.type.rust_type_name()}", bits = {self.bit_length})]
enum {self.name} {{
{self._variants_to_rust()}
}}'''

    def _variants_to_rust(self) -> str:
        variants = [var.to_rust() for var in self.variants]
        return '\n'.join([indent(var) for var in variants])


class RustTypeCache:
    def __init__(self) -> None:
        self.struct_cache: Dict[str, RustStruct] = {}
        self.enum_cache: Dict[str, RustEnum] = {}
        self.unresolved_structs: List[Tuple[RustStruct, elt.Envelope]] = []

    def get_rust_struct(self, pyobj: elt.Envelope, add_to_unresolved = True) -> RustStruct:
        if pyobj._name in self.struct_cache:
            return self.struct_cache[pyobj._name]
        rust_struct = RustStruct.from_pycrate(pyobj)
        if add_to_unresolved:
            self.unresolved_structs.append((rust_struct, pyobj))
        self.struct_cache[rust_struct.name] = rust_struct
        return rust_struct

    def resolve_struct(self):
        rust_struct, pyobj = self.unresolved_structs.pop()
        print('resolving', rust_struct.name, pyobj)
        bit_padding = None
        for field in pyobj:
            bit_length = None
            if isinstance(field, elt.Atom):
                if isinstance(field, Buf):
                    rust_type = RustPrimitiveType.VecU8
                    bit_length = field.get_bl()
                elif field._dic:
                    rust_enum = self.get_rust_enum(field, rust_struct.name)
                    bit_length = field.get_bl()
                    rust_type = rust_enum
                else:
                    rust_type = RustPrimitiveType.from_pycrate(field)
                    bit_length = field.get_bl()
            elif isinstance(field, elt.Envelope):
                rust_type = self.get_rust_struct(field)
            else:
                rust_type = None
            field = RustStructField(
                field._name,
                rust_type,
                None,
                bit_length,
                bit_padding,
            )
            rust_struct.add_field(field)

    def get_rust_enum(self, pyobj: Any, prefix: str) -> RustEnum:
        name = prefix + pyobj._name
        if name in self.enum_cache:
            return self.enum_cache[name]
        rust_enum = RustEnum.from_pycrate(pyobj, prefix)
        self.enum_cache[rust_enum.name] = rust_enum
        return rust_enum


class RustModule:
    def __init__(self, pyobj: Layer3E) -> None:
        self.cache = RustTypeCache()
        self.pyobj = pyobj
        self.base_struct = self.cache.get_rust_struct(pyobj, False)

    def resolve_types(self) -> None:
        bit_padding = None

        # skip the EMMHeader
        assert isinstance(self.pyobj._GEN[0], EMMHeader)
        for item in self.pyobj._GEN[1:]:
            layer3_wrapper = get_layer3_wrapper(item)

            # the only time we don't have a layer 3 TLV is bit padding
            if layer3_wrapper is None:
                assert isinstance(item, Uint)
                assert item._name == 'spare'
                bit_padding = item.get_bl()
                continue

            # prepare the layer 3 TLV's inner value
            if item._IE_stat is not None:
                inner = item._IE_stat
                # check for unsupported types (these will be NoneType)
                if isinstance(inner, (
                    elt.Sequence,
                    elt.Array,
                    CSN1List,
                    LCSClientId,
                )):
                    field = RustStructField(
                        item._name,
                        None,
                        layer3_wrapper,
                        None,
                        bit_padding,
                    )
                    self.base_struct.add_field(field)
                    continue
                # check for Buf-type IEs
                if isinstance(inner, elt.Atom):
                    field = RustStructField(
                        item._name,
                        RustPrimitiveType.from_pycrate(inner),
                        layer3_wrapper,
                        None,
                        bit_padding
                    )
                    self.base_struct.add_field(field)
                    continue
                field_struct = self.cache.get_rust_struct(inner)
                field = RustStructField(
                    item._name,
                    field_struct,
                    layer3_wrapper,
                    None,
                    bit_padding,
                )
                self.base_struct.add_field(field)
            else:
                inner = item._V
                if inner._dic is not None:
                    field_enum = self.cache.get_rust_enum(inner, item._name)
                    field = RustStructField(
                        item._name,
                        field_enum,
                        layer3_wrapper,
                        None,
                        bit_padding,
                    )
                    self.base_struct.add_field(field)
                else:
                    field = RustStructField(
                        item._name,
                        None,
                        layer3_wrapper,
                        None,
                        bit_padding,
                    )
                    self.base_struct.add_field(field)

        while len(self.cache.unresolved_structs):
            self.cache.resolve_struct()

    def to_rust(self) -> str:
        emm_header_names = [
            'EMMHeaderProtDisc',
            'EMMHeaderSecHdr',
            'EMMHeaderType',
        ]
        structs = [struct for name, struct in self.cache.struct_cache.items() if name != 'EMMHeader']
        enums = [enum for name, enum in self.cache.enum_cache.items() if name not in emm_header_names]
        return f"""
use deku::prelude::*;
use crate::nas::layer3::*;

{'\n\n'.join([rust_struct.to_rust() for rust_struct in structs])}
{'\n\n'.join([rust_enum.to_rust() for rust_enum in enums])}
"""


def main():
    for i in NASLTE.EMMTypeMOClasses:
        obj = NASLTE.EMMTypeMOClasses[i]()
        module = RustModule(obj)
        module.resolve_types()
        print(module.to_rust())


if __name__ == "__main__":
    main()
