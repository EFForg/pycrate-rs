import binascii
import os
from typing import Dict, List, Tuple, Any, Type
from pycrate_core import elt
from pycrate_core.base import Uint, Buf
from pycrate_csn1.csnobj import CSN1List
from pycrate_mobile import NASLTE
from pycrate_mobile.TS24301_EMM import EMMHeader
from pycrate_mobile.TS24007 import Layer3E
from pycrate_mobile.TS24301_ESM import ESMHeader
from pycrate_mobile.TS24301_IE import LCSClientId

from generator.rust_types import RustStruct, RustEnum, RustPrimitiveType, RustStructField, get_layer3_wrapper
from generator.tests import RustTestCase


class RustTypeCache:
    """Keeps track of generated Rust types based on the pycrate object that
    created them."""

    def __init__(self) -> None:
        self.struct_cache: Dict[str, RustStruct] = {}
        self.enum_cache: Dict[str, RustEnum] = {}
        self.unresolved_structs: List[Tuple[RustStruct, elt.Envelope]] = []

    def get_rust_struct(
        self,
        pyobj: elt.Envelope,
        add_to_unresolved=True,
    ) -> RustStruct:
        """Get (or create) a RustStruct for the given pycrate object. RustStructs
        created this way are by default pushed onto the stack of unresolved
        structs
        """
        if pyobj._name in self.struct_cache:
            return self.struct_cache[pyobj._name]
        rust_struct = RustStruct.from_pycrate(pyobj)
        if add_to_unresolved:
            self.unresolved_structs.append((rust_struct, pyobj))
        self.struct_cache[rust_struct.name] = rust_struct
        return rust_struct

    def resolve_struct(self):
        """Pop an unresolved struct off the stack and for each of its fields,
        generate either a primitive type, enum, or struct. Other structs
        generated this way are pushed onto the unresolved_structs stack.
        """
        rust_struct, pyobj = self.unresolved_structs.pop()
        bit_padding = None
        for i, item in enumerate(pyobj):
            bit_length = None
            is_final_buf = False
            if isinstance(item, elt.Atom):
                if isinstance(item, Buf):
                    rust_type = RustPrimitiveType.VecU8
                    bit_length = item.get_bl()
                    # if the given bitlength is 0, we're assuming that this
                    # buffer is the final field in the struct, and ought to
                    # consume all remaining bytes
                    if bit_length == 0:
                        is_final_buf = True
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
            rust_field.is_final_buf = is_final_buf
            rust_struct.add_field(rust_field, i)

    def get_rust_enum(self, pyobj: Any, prefix: str) -> RustEnum:
        """Get (or create) a RustEnum for the given pycrate object"""
        name = prefix + pyobj._name
        if name in self.enum_cache:
            return self.enum_cache[name]
        rust_enum = RustEnum.from_pycrate(pyobj, prefix)
        self.enum_cache[rust_enum.name] = rust_enum
        return rust_enum


class RustModule:
    """A Rust module derived from a single pycrate class."""

    def __init__(self, pyobj: Layer3E) -> None:
        self.cache = RustTypeCache()
        self.pyobj = pyobj

        # don't mark the base struct as unresolved, since we'll be manually
        # resolving it later
        self.base_struct = self.cache.get_rust_struct(self.pyobj, False)
        self.name = self.base_struct.name.lower()
        self.test_cases: list[RustTestCase] = []

    def resolve_types(self) -> None:
        """For every field in the pycrate class, generate a RustStruct or
        RustEnum wrapped in the corresponding Layer3Wrapper. As these types are
        generated, they'll be added to the module's RustTypeCache to be fully
        resolved later.
        """
        bit_padding = None

        for i, item in enumerate(self.pyobj._content):
            # skip the header
            if i == 0:
                assert isinstance(item, (EMMHeader, ESMHeader))
                continue

            # the only time we don't have a layer 3 TLV is bit padding
            layer3_wrapper = get_layer3_wrapper(item)
            if layer3_wrapper is None:
                assert isinstance(item, Uint)
                assert item._name == 'spare'
                bit_padding = item.get_bl()
                continue

            # prepare the layer 3 TLV's inner value. depending on the type of
            # layer 3 TLV, sometimes this is stored in _IE_stat, sometimes in
            # _V
            if item._IE_stat is not None:
                inner = item._IE_stat
            else:
                inner = item._V

            bit_length = None if layer3_wrapper.type.is_sized() else inner.get_bl()
            # check for unsupported types
            if isinstance(inner, (
                elt.Sequence,
                elt.Array,
                CSN1List,
                LCSClientId,
            )):
                # passing None for type results in the Rust value being a
                # unit type, aka `()`
                field = RustStructField(
                    item._name,
                    None,
                    layer3_wrapper,
                    bit_length,
                    bit_padding,
                )
                bit_padding = None
                self.base_struct.add_field(field, None)
                continue
            if isinstance(inner, elt.Atom):
                # check for enums
                if inner._dic is not None:
                    field_enum = self.cache.get_rust_enum(inner, item._name)
                    field = RustStructField(
                        item._name,
                        field_enum,
                        layer3_wrapper,
                        bit_length,
                        bit_padding,
                    )
                    bit_padding = None
                    self.base_struct.add_field(field, i)
                    continue
                field = RustStructField(
                    item._name,
                    RustPrimitiveType.from_pycrate(inner),
                    layer3_wrapper,
                    bit_length,
                    bit_padding
                )
                bit_padding = None
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
            bit_padding = None

        while len(self.cache.unresolved_structs):
            self.cache.resolve_struct()

    def add_test_case(self, input_hexstring: str, input_bytes: bytes) -> None:
        """Given some input bytes, generate a RustTestCase which asserts
        equality for every parsed field
        """

        # the pycrate method from_bytes() sets all of the objects internal
        # values according to the binary payload, and since we've associated
        # each rust element with a corresponding pycrate element (or index into
        # element), this lets us easily match rust values to the expected parsed
        # pycrate value
        x = self.pyobj.__class__()
        x.from_bytes(input_bytes)
        name = f'case_{len(self.test_cases) + 1}'
        self.test_cases.append(RustTestCase(
            name,
            input_hexstring,
            self.base_struct,
            x
        ))

    def _tests_to_rust(self) -> str:
        total_assertions = sum(len(c.assertions) for c in self.test_cases)
        if total_assertions == 0:
            if len(self.test_cases):
                print(f'warning: {self.name} has test cases but no assertions!')
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
        """Generates Rust code for this module's struct and enums, along with
        any tests cases.
        """
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


class RustModuleIndex:
    """Contains a number of RustModules, allowing us to output a mod.rs which
    declares them all
    """

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
        os.makedirs(filepath, exist_ok=True)
        index_path = os.path.join(filepath, 'mod.rs')
        with open(index_path, 'w') as f:
            f.write(self.to_rust())

        for mod in self.modules:
            mod_path = os.path.join(filepath, f'{mod.name}.rs')
            with open(mod_path, 'w') as f:
                f.write(mod.to_rust())


def generate_module(filepath: str, classes: list[Type[Layer3E]], test_cases: list[str]=[]) -> None:
    """Given a set of pycrate classes, creates a directory containing a Rust
    module for each class, as well as a mod.rs file declaring each of them. Also
    appends a standard Rust unit test section to each module for each test case
    provided.
    """
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
