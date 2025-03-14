import binascii
import os
from typing import Dict, List, Optional, Tuple, Union, Any, Type
from pycrate_core import elt
from pycrate_core.base import Uint, Buf, Uint8, Uint16
from pycrate_csn1.csnobj import CSN1List
from pycrate_mobile import NASLTE
from pycrate_mobile.TS24007 import IE, Layer3E
from pycrate_mobile.TS24301_EMM import EMMHeader
from pycrate_mobile.TS24301_ESM import ESMHeader
from enum import StrEnum, IntEnum, auto
from pycrate_mobile.TS24301_IE import LCSClientId
from namer import Name


RESERVED_WORDS = [
    'type',
]


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


def upper_camel_case(s: str) -> str:
    return Name(s).cc()


def snake_case(s: str) -> str:
    return Name(s).sc()


def derives(partial_eq=False) -> str:
    traits = [
        'DekuRead',
        # 'DekuWrite', # TODO: implement DekuWrite for
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

    def _deku_attrs(self) -> str:
        deku_attrs = []
        # if we have a bit_length and we're not an enum, add a deku attr
        # declaring it. (enum bitlengths go on the enum decl)
        if self.bit_length is not None and not isinstance(self.type, RustEnum):
            if self.type == RustPrimitiveType.VecU8:
                assert self.bit_length % 8 == 0
                byte_length = int(self.bit_length / 8)
                deku_attrs.append(f'count = "{byte_length}"')
            elif self.bit_length % 8 == 0:
                deku_attrs.append(f'bytes = {int(self.bit_length / 8)}')
            else:
                deku_attrs.append(f'bits = {self.bit_length}')
        if self.bit_padding is not None:
            deku_attrs.append(f'pad_bits_before = "{self.bit_padding}"')
        if self.type is not None and self.type.is_big_endian():
            deku_attrs.append('endian = "big"')
        if self.is_optional:
            deku_attrs.append('cond = "deku::byte_offset < byte_size"')
            if isinstance(self.type, RustEnum):
                default_name = f"{self.type.name}::{self.type.variants[0].name}"
                deku_attrs.append(f'default = "{default_name}"')
        ctx = []
        is_layer3_buffer = False
        if self.layer3_wrapper is not None:
            if self.layer3_wrapper.tag is not None:
                ctx.append(f'Tag({self.layer3_wrapper.tag})')
            if self.type == RustPrimitiveType.VecU8:
                is_layer3_buffer = True
        is_variable_bitfield = isinstance(self.type, RustStruct) and self.type.is_variable_bitfield
        if is_variable_bitfield or is_layer3_buffer:
            ctx.append("NeedsByteSize")
        if len(ctx):
            deku_attrs.append(f'ctx = "{', '.join(ctx)}"')
        deku_part = ''
        if len(deku_attrs):
            deku_part = f'#[deku({', '.join(deku_attrs)})] '
        return deku_part

    def to_rust(self) -> str:
        # special case for Type4TLV<Vec<u8>>
        type_name = '()' if self.type is None else self.type.rust_type_name()
        if self.layer3_wrapper is not None:
            if self.layer3_wrapper.type == Layer3Type.Type4TLV:
                if self.type == RustPrimitiveType.VecU8:
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

    def to_rust(self) -> str:
        self._fix_all_duplicates()
        deku_ctx = ''
        if self.is_variable_bitfield:
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


def indent(s: str, num_indents=1) -> str:
    indentation = ' ' * 4 * num_indents
    lines = [indentation + line for line in s.split('\n')]
    return '\n'.join(lines)


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


class RustTypeCache:
    def __init__(self) -> None:
        self.struct_cache: Dict[str, RustStruct] = {}
        self.enum_cache: Dict[str, RustEnum] = {}
        self.unresolved_structs: List[Tuple[RustStruct, elt.Envelope]] = []

    # Get (or create) a RustStruct for the given pycrate object. RustStructs
    # created this way are by default pushed onto the stack of unresolved
    # structs
    def get_rust_struct(
        self,
        pyobj: elt.Envelope,
        add_to_unresolved=True,
    ) -> RustStruct:
        if pyobj._name in self.struct_cache:
            return self.struct_cache[pyobj._name]
        rust_struct = RustStruct.from_pycrate(pyobj)
        if add_to_unresolved:
            self.unresolved_structs.append((rust_struct, pyobj))
        self.struct_cache[rust_struct.name] = rust_struct
        return rust_struct

    # Pop an unresolved struct off the stack and for each of its fields,
    # generate either a primitive type, enum, or struct. Other structs
    # generated this way are pushed onto the unresolved_structs stack.
    def resolve_struct(self):
        rust_struct, pyobj = self.unresolved_structs.pop()
        bit_padding = None
        for i, item in enumerate(pyobj):
            bit_length = None
            if isinstance(item, elt.Atom):
                if isinstance(item, Buf):
                    rust_type = RustPrimitiveType.VecU8
                    bit_length = item.get_bl()
                elif item._dic:
                    rust_enum = self.get_rust_enum(item, rust_struct.name)
                    bit_length = item.get_bl()
                    rust_type = rust_enum
                else:
                    rust_type = RustPrimitiveType.from_pycrate(item)
                    bit_length = item.get_bl()
            elif isinstance(item, elt.Envelope):
                rust_type = self.get_rust_struct(item)
            else:
                rust_type = None
            rust_field = RustStructField(
                item._name,
                rust_type,
                None,
                bit_length,
                bit_padding,
            )
            rust_struct.add_field(rust_field, i)

    # Get (or create) a RustEnum for the given pycrate object
    def get_rust_enum(self, pyobj: Any, prefix: str) -> RustEnum:
        name = prefix + pyobj._name
        if name in self.enum_cache:
            return self.enum_cache[name]
        rust_enum = RustEnum.from_pycrate(pyobj, prefix)
        self.enum_cache[rust_enum.name] = rust_enum
        return rust_enum


# A rust module derived from a single pycrate class.
class RustModule:
    def __init__(self, pyobj: Layer3E) -> None:
        self.cache = RustTypeCache()
        self.pyobj = pyobj

        # don't mark the base struct as unresolved, since we'll be manually
        # resolving it later
        self.base_struct = self.cache.get_rust_struct(self.pyobj, False)
        self.name = self.base_struct.name.lower()
        self.test_cases = []

    def resolve_types(self) -> None:
        bit_padding = None

        for i, item in enumerate(self.pyobj._content):
            # skip the EMMHeader
            if i == 0:
                assert isinstance(item, (EMMHeader, ESMHeader))
                continue

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
                bit_length = None if layer3_wrapper.type.is_sized() else inner.get_bl()
                # check for unsupported types
                if isinstance(inner, (
                    elt.Sequence,
                    elt.Array,
                    CSN1List,
                    LCSClientId,
                )):
                    # passing None for type results in an inner type of unit, aka `()`
                    field = RustStructField(
                        item._name,
                        None,
                        layer3_wrapper,
                        bit_length,
                        bit_padding,
                    )
                    self.base_struct.add_field(field, None)
                    continue
                # check for Buf-type IEs
                if isinstance(inner, elt.Atom):
                    field = RustStructField(
                        item._name,
                        RustPrimitiveType.from_pycrate(inner),
                        layer3_wrapper,
                        bit_length,
                        bit_padding
                    )
                    self.base_struct.add_field(field, i)
                    continue
                field_struct = self.cache.get_rust_struct(inner)
                field = RustStructField(
                    item._name,
                    field_struct,
                    layer3_wrapper,
                    bit_length,
                    bit_padding,
                )
                self.base_struct.add_field(field, i)
            else:
                inner = item._V
                bit_length = None if layer3_wrapper.type.is_sized() else inner.get_bl()
                if inner._dic is not None:
                    field_enum = self.cache.get_rust_enum(inner, item._name)
                    field = RustStructField(
                        item._name,
                        field_enum,
                        layer3_wrapper,
                        bit_length,
                        bit_padding,
                    )
                    self.base_struct.add_field(field, i)
                else:
                    field = RustStructField(
                        item._name,
                        None,
                        layer3_wrapper,
                        bit_length,
                        bit_padding,
                    )
                    self.base_struct.add_field(field, None)
            bit_padding = None

        while len(self.cache.unresolved_structs):
            self.cache.resolve_struct()

    def add_test_case(self, input_hexstring: str, input_bytes: bytes) -> None:
        # the pycrate method from_bytes() sets all of the objects internal
        # values according to the binary payload, and since we've associated
        # each rust element with a corresponding pycrate element (or index into
        # element), this lets us easily match rust values to the expected parsed
        # pycrate value
        self.pyobj.from_bytes(input_bytes)
        name = f'case_{len(self.test_cases) + 1}'
        self.test_cases.append(RustTestCase(name, input_hexstring, self.base_struct, self.pyobj))

    def _tests_to_rust(self) -> str:
        if len(self.test_cases) == 0:
            return ''
        return f'''
#[cfg(test)]
mod tests {{
    use super::*;
    use crate::nas::test_utils::*;
    use deku::prelude::*;
    use std::io::Cursor;

{'\n\n'.join([test_case.to_rust() for test_case in self.test_cases])}
}}
'''

    def to_rust(self) -> str:
        excluded_structs = [
            'EMMHeader',
            'ESMHeader',
        ]
        emm_header_names = [
            'EMMHeaderProtDisc',
            'EMMHeaderSecHdr',
            'EMMHeaderType',
        ]
        esm_header_names = [
            'ESMHeaderESPBearerId',
            'ESMHeaderProtDisc'
            'ESMHeaderPTI',
            'ESMHeaderType',
        ]
        excluded_enums = emm_header_names + esm_header_names
        structs = [struct for name, struct in self.cache.struct_cache.items() if name not in excluded_structs]
        enums = [enum for name, enum in self.cache.enum_cache.items() if name not in excluded_enums]
        test_cases = self._tests_to_rust()
        return f"""
use deku::prelude::*;
use deku::ctx::ByteSize;
use serde::Serialize;
use crate::nas::layer3::*;

{'\n\n'.join([rust_struct.to_rust() for rust_struct in structs])}
{'\n\n'.join([rust_enum.to_rust() for rust_enum in enums])}
{test_cases}
"""


class RustTestCaseValue:
    def __init__(
        self,
        typ: RustEnum | RustPrimitiveType,
        value: int | str | bytes,
    ) -> None:
        self.type = typ
        self.value = value
        self.matching_enum_variant = None
        if isinstance(typ, RustEnum):
            for var in typ.variants:
                if value in var.values:
                    self.matching_enum_variant = var
        if typ == RustPrimitiveType.VecU8:
            assert isinstance(value, bytes)

    def to_rust(self) -> str:
        if isinstance(self.type, RustEnum):
            if self.matching_enum_variant is None:
                # FIXME: once https://github.com/sharksforarms/deku/issues/533
                # is fixed, we can match on the catchall variant value
                # return f'{self.type.name}::Other({self.value})'
                return f'{self.type.name}::Other'
            else:
                return f'{self.type.name}::{self.matching_enum_variant.name}'
        elif self.type == RustPrimitiveType.VecU8:
            assert isinstance(self.value, bytes)
            byte_ints = [str(int(b)) for b in self.value]
            return f'vec![{', '.join(byte_ints)}]'
        else:
            return f'{self.value}'


class RustTestCase:
    def __init__(self, name: str, input_hexstring: str, struct: RustStruct, pyobj: elt.Envelope) -> None:
        self._input_hexstring = input_hexstring
        self.name = name
        self.struct = struct
        self.assertions = self._build_assertions([], struct, pyobj)

    def _build_assertions(
        self,
        ancestors: list[RustStructField],
        struct: RustStruct,
        pyobj: elt.Envelope,
    ) -> list[Tuple[list[RustStructField], RustTestCaseValue]]:
        assertions = []
        for i in range(len(struct.fields)):
            field = struct.fields[i]
            # skip values we don't have a parser for, as well as spare bits
            if field.type is None or field.name.startswith('spare'):
                continue
            pyobj_index = struct.pyobj_indices[i]
            if pyobj_index is None:
                continue
            item = pyobj[pyobj_index]
            # if a value doesn't appear in the payload, pycrate sets a
            # "transparency" flag. skip these since our rust object won't
            # have values for these either
            if item.get_trans():
                continue

            # if the pyobj is in a layer3 wrapper, unwrap it so we can index it
            # correctly
            layer3_wrapper = get_layer3_wrapper(item)
            if layer3_wrapper is not None:
                item = item[layer3_wrapper.get_inner_idx()]
            fields = ancestors + [field]

            # if we're on a concrete value like a number, buffer, or enum,
            # simply add the assertion. otherwise, we're on a struct, so recurse
            if isinstance(field.type, (RustPrimitiveType, RustEnum)):
                assertions.append(
                    (fields, RustTestCaseValue(field.type, item.get_val()))
                )
            else:
                assert isinstance(item, elt.Envelope)
                assertions += self._build_assertions(fields, field.type, item)

        return assertions

    def _assertions_to_rust(self, ident_name: str) -> str:
        lines = []
        unwrapped_layer3_idents = []
        for (fields, value) in self.assertions:
            assert fields[0].layer3_wrapper is not None
            layer3_ident = fields[0].name
            if layer3_ident not in unwrapped_layer3_idents:
                inner_part = f'{fields[0].name}.inner'
                if fields[0].layer3_wrapper.type.is_tagged():
                    inner_part += ".as_ref().unwrap()"
                lines.append(f'let {layer3_ident} = {ident_name}.{inner_part};')
                unwrapped_layer3_idents.append(layer3_ident)
            field_train = '.'.join([field.name for field in fields[1:]])
            lhs = f'{layer3_ident}.{field_train}'
            rhs = value.to_rust()
            lines.append(f'assert_eq!({lhs}, {rhs});')
        return '\n'.join(lines)

    def to_rust(self) -> str:
        ident_name = 'msg'
        # skip the header (first two bytes)
        test_case_bytes = self._input_hexstring[4:]
        return indent(f'''#[test]
fn test_{self.name}() {{
    let mut bytes = Cursor::new(unhexlify("{test_case_bytes}"));
    let mut reader = Reader::new(&mut bytes);
    let {ident_name} = {self.struct.name}::from_reader_with_ctx(&mut reader, ())
        .expect("failed to parse");
{indent(self._assertions_to_rust(ident_name))}
}}''')


class RustModuleIndex:
    def __init__(self) -> None:
        self.modules: List[RustModule] = []

    def add(self, module: RustModule) -> None:
        self.modules.append(module)

    def to_rust(self) -> str:
        module_text = '\n'.join(f'pub mod {mod.name};' for mod in self.modules)
        return f"""
#![allow(unused_imports)]

{module_text}"""

    def generate_module(self, filepath: str) -> None:
        index_path = os.path.join(filepath, 'mod.rs')
        print(f'writing index to {index_path}')
        with open(index_path, 'w') as f:
            f.write(self.to_rust())

        for mod in self.modules:
            mod_path = os.path.join(filepath, f'{mod.name}.rs')
            print(f'writing {mod.name} to {mod_path}')
            with open(mod_path, 'w') as f:
                f.write(mod.to_rust())


def generate_module(filepath: str, classes: list[Type[Layer3E]], test_cases: list[str]=[]) -> None:
    index = RustModuleIndex()
    pycrate_names_to_modules = {}
    for clazz in classes:
        obj = clazz()
        module = RustModule(obj)
        module.resolve_types()
        pycrate_names_to_modules[obj._name] = module
        index.add(module)

    # generate test cases
    for case_str in test_cases:
        # first, parse the payload in pycrate to determine which module this
        # will be added to
        case = binascii.unhexlify(case_str)
        # we don't know apriori whether this is MT or MO, so try both
        m, e = NASLTE.parse_NASLTE_MO(case)
        if e != 0:
            m, e = NASLTE.parse_NASLTE_MT(case)
            print(case_str, case, m, e)
            assert e == 0
        rust_module = pycrate_names_to_modules[m._name]

        # now add the test case
        rust_module.add_test_case(case_str, case)

    index.generate_module(filepath)


def main(filepath: str):
    emm_classes = list(NASLTE.EMMTypeMOClasses.values())
    emm_classes.append(NASLTE.EMMTypeMTClasses[69])  # add in the MT version of DetachRequest
    generate_module(os.path.join(filepath, 'emm'), emm_classes, [
        '0748610bf602f8108003c8c2e65e9a5804e060c0405202f810c4c25c0a00570220003103e5e0341302f810040511035758a65d0100c1', # EMM TAU Request
    ])
    generate_module(os.path.join(filepath, 'esm'), NASLTE.ESMTypeClasses.values(), [
        '0202d9', # ESM Info Req
    ])


if __name__ == "__main__":
    import sys
    main(sys.argv[1])
