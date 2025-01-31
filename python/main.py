from typing import List, Optional, Union
from pycrate_core import elt
from pycrate_mobile import NASLTE
from pycrate_mobile.TS24007 import IE
from enum import StrEnum, IntEnum, auto


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
    if ' ' in s:
        return s.title().replace(' ', '')
    else:
        return s


def snake_case(s: str) -> str:
    snake_cased = s.replace(' ', '_').lower()
    if snake_cased in RESERVED_WORDS:
        snake_cased = snake_cased[:-1]
    return snake_cased


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
        type_name = type(obj).__name__
        return {
            'Uint': RustPrimitiveType.U32,
            'Uint8': RustPrimitiveType.U8,
            'Buf': RustPrimitiveType.VecU8
        }[type_name]


class RustStructField:
    def __init__(
        self,
        name: str,
        type: 'RustPrimitiveType | RustStruct | RustEnum',
        bit_length: Optional[int],
    ):
        self.type = type
        self.name = snake_case(name)
        self.bit_length = bit_length

    def to_rust(self) -> str:
        type_name = self.type.rust_type_name()
        if not isinstance(self.type, RustPrimitiveType) and self.type.layer3_wrapper is not None:
            wrapper_name = str(self.type.layer3_wrapper)
            type_name = f"{wrapper_name}<{type_name}>"
        # deku bits attribute goes on the enum decl
        if self.bit_length is None or isinstance(self.type, RustEnum):
            deku_part = ''
        else:
            deku_part = f'#[deku(bits = {self.bit_length})] '
        return f'{deku_part}{self.name}: {type_name},'


class RustStruct:
    def __init__(self, name: str, layer3_wrapper: Optional[Layer3Wrapper] = None) -> None:
        self.fields: List[RustStructField] = []
        self.layer3_wrapper = layer3_wrapper
        self.name = upper_camel_case(name)

    @staticmethod
    def from_pycrate(obj: elt.Envelope, prefix: Optional[str]) -> 'RustStruct':
        name = obj._name
        if prefix:
            name = prefix + name
        return RustStruct(name, get_layer3_wrapper(obj))

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
        self.value = value

    def to_rust(self) -> str:
        return f'#[deku(id_pat = {self.value})] {self.name},'


class RustEnum:
    def __init__(self, name: str, type: RustPrimitiveType, bit_length: int) -> None:
        self.variants: List[RustEnumVariant] = []
        self.type = type
        self.bit_length = bit_length
        self.layer3_wrapper: Optional[Layer3Wrapper] = None
        self.name = upper_camel_case(name)

    @staticmethod
    def from_pycrate(obj: elt.Atom, prefix: Optional[str]) -> 'RustEnum':
        name = obj._name
        if prefix:
            name = prefix + obj._name
        rust_enum = RustEnum(
            name,
            RustPrimitiveType.from_pycrate(obj),
            obj.get_bl(),
        )
        assert obj._dic is not None
        for val, name in obj._dic.items():
            rust_enum.add_variant(RustEnumVariant(name, val))
        return rust_enum

    def add_variant(self, variant: RustEnumVariant):
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


class NoneType:
    def __init__(self, layer3_wrapper: Layer3Wrapper):
        self.layer3_wrapper = layer3_wrapper

    def rust_type_name(self) -> str:
        return 'NoneType'


class RustModule:
    def __init__(self, name: str) -> None:
        self.items: List[Union[RustStruct, RustEnum]] = []

    def _get_pycrate_field(
        self,
        obj: elt.Atom | elt.Envelope,
        prefix: str
    ) -> RustStructField:
        print('field', type(obj), obj._name)
        bit_length = None
        rust_type: RustPrimitiveType | RustEnum | RustStruct
        if isinstance(obj, elt.Atom):
            if obj._dic:
                rust_enum = RustEnum.from_pycrate(obj, prefix)
                self.add_item(rust_enum)
                bit_length = obj.get_bl()
                rust_type = rust_enum
            else:
                rust_type = RustPrimitiveType.from_pycrate(obj)
                bit_length = obj.get_bl()
        elif isinstance(obj, elt.Envelope):
            rust_type = self.add_pycrate_obj(obj, prefix)
        return RustStructField(
            obj._name,
            rust_type,
            bit_length,
        )

    def add_pycrate_obj(self, obj: elt.Atom | elt.Envelope, prefix: Optional[str] = None) -> RustEnum | RustStruct | NoneType:
        print('obj', type(obj), obj._name)
        if isinstance(obj, elt.Atom):
            if obj._dic:
                return self.add_item(RustEnum.from_pycrate(obj, prefix))
            else:
                raise ValueError(f'toplevel primitive obj {obj}')
        elif isinstance(obj, IE):
            layer3_wrapper = get_layer3_wrapper(obj)
            assert layer3_wrapper is not None
            print('wrapper', layer3_wrapper)
            inner = obj._IE_stat
            if inner is None:
                inner = obj._V
            if isinstance(inner, elt.Atom) or isinstance(inner, elt.Sequence):
                return NoneType(layer3_wrapper)
            item = self.add_pycrate_obj(inner, prefix)
            assert not isinstance(item, NoneType)
            item.name = obj._name
            item.layer3_wrapper = layer3_wrapper
            return item
        elif isinstance(obj, elt.Envelope):
            rust_struct = RustStruct.from_pycrate(obj, prefix)
            for sub in obj:
                rust_struct.add_field(self._get_pycrate_field(sub, rust_struct.name))
            return self.add_item(rust_struct)
        else:
            raise ValueError(f'unknown obj {obj}')

    def add_item(self, item: RustStruct | RustEnum) -> RustStruct | RustEnum:
        for existing_item in self.items:
            if item.name == existing_item.name:
                raise ValueError(f'Item with name {item.name} already exists')
        self.items.append(item)
        return item

    def to_rust(self) -> str:
        return f'''\
use deku::prelude::*;
use crate::nas::layer3::*;

{self._items_to_rust()}'''

    def _items_to_rust(self) -> str:
        return '\n\n'.join([item.to_rust() for item in self.items])


def main():
    for i in NASLTE.EMMTypeMOClasses:
        obj = NASLTE.EMMTypeMOClasses[i]()
        module = RustModule('foo')
        module.add_pycrate_obj(obj)
        print(module.to_rust())


if __name__ == "__main__":
    main()
