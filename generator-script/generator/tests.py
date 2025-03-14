from typing import Tuple
from pycrate_core import elt

from generator.rust_types import RustStruct, RustEnum, RustPrimitiveType, RustStructField, get_layer3_wrapper
from generator.util import indent


class RustTestCaseValue:
    """Represents a Rust literal value which'll be used in the right-hand side
    of an assert_eq!() comparison.
    """

    def __init__(
        self,
        typ: RustEnum | RustPrimitiveType,
        value: int | str | bytes,
    ) -> None:
        self.type = typ
        self.value = value

        # if we're representing an enum's value, try to find an exact match
        # for the value in the enum's variants. if we can't, it'll be matched
        # against the catchall Other variant
        self.matching_enum_variant = None
        if isinstance(typ, RustEnum):
            for var in typ.variants:
                if value in var.values:
                    self.matching_enum_variant = var
        # make sure we're comparing apples to apples
        if typ == RustPrimitiveType.VecU8:
            assert isinstance(value, bytes)

    def to_rust(self) -> str:
        """Generates Rust code representing this literal value"""

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
            assert not isinstance(self.value, bytes)
            return f'{self.value}'


class RustTestCase:
    """Represents a single test function within a module's unit tests. Each
    RustTestCase parses its input hexstring directly to the given RustStruct,
    then provides a long list of equality assertions for every single field
    detected in the resulting pycrate object.
    """

    def __init__(
        self,
        name: str,
        input_hexstring: str,
        struct: RustStruct,
        pyobj: elt.Envelope
    ) -> None:
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

            # recover this field's corresponding pycrate object, skipping if we
            # don't find one
            pyobj_index = struct.pyobj_indices[i]
            if pyobj_index is None:
                continue
            item = pyobj[pyobj_index]

            # if a value doesn't appear in the payload, pycrate sets its
            # "transparency" flag. skip these since our rust object won't have
            # values for these either
            if item.get_trans():
                continue

            # if the pyobj is in a layer3 wrapper, unwrap it so we can index it
            # correctly
            layer3_wrapper = get_layer3_wrapper(item)
            if layer3_wrapper is not None:
                item = item[layer3_wrapper.get_inner_idx()]

            # append the current field to the list of fields
            fields = ancestors + [field]

            # if we're on a literal value like a number, buffer, or enum,
            # simply add the assertion
            if isinstance(field.type, (RustPrimitiveType, RustEnum)):
                assertions.append(
                    (fields, RustTestCaseValue(field.type, item.get_val()))
                )
            else:
                # otherwise, we're on a struct, so recurse
                assert isinstance(item, elt.Envelope)
                assertions += self._build_assertions(fields, field.type, item)

        return assertions

    def _assertions_to_rust(self, ident_name: str) -> str:
        lines = []
        unwrapped_layer3_idents = []
        for (fields, value) in self.assertions:
            # every root field should be in a layer 3 container
            assert fields[0].layer3_wrapper is not None

            # if this is the first assertion referencing the topmost field,
            # pull it out since fetching the inner value in every line gets
            # pretty verbose
            layer3_ident = fields[0].name
            if layer3_ident not in unwrapped_layer3_idents:
                inner_part = f'{fields[0].name}.inner'
                # if it's a tagged layer 3 container, `.inner` is an Option<T>,
                # so pull out a reference to it
                if fields[0].layer3_wrapper.type.is_tagged():
                    inner_part += ".as_ref().unwrap()"

                # results in code like `let foo = msg.foo.inner;`, letting
                # subsequent assertions just compare against `foo`
                lines.append(f'let {layer3_ident} = {ident_name}.{inner_part};')
                unwrapped_layer3_idents.append(layer3_ident)

            subfield_names = [field.name for field in fields[1:]]
            lhs = '.'.join([layer3_ident] + subfield_names)
            rhs = value.to_rust()
            lines.append(f'assert_eq!({lhs}, {rhs});')
        return '\n'.join(lines)

    def to_rust(self) -> str:
        """Generates the unit test function, to be held within a #[cfg(test)]
        module.
        """

        ident_name = 'msg'
        # skip the NAS header (first two bytes)
        test_case_bytes = self._input_hexstring[4:]
        return indent(f'''#[test]
fn test_{self.name}() {{
    let mut bytes = Cursor::new(unhexlify("{test_case_bytes}"));
    let mut reader = Reader::new(&mut bytes);
    let {ident_name} = {self.struct.name}::from_reader_with_ctx(&mut reader, ())
        .expect("failed to parse");
{indent(self._assertions_to_rust(ident_name))}
}}''')
