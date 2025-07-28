# pycrate-rs

This repo consists of two projects: a NAS telcom message parser written in rust, and the python script which generates it by relying on the [pycrate](https://github.com/pycrate-org/pycrate/) project.

## Generating the parser/tests

From a clean repo:

```
$ python -m venv venv
$ . venv/bin/activate
$ pip install ./generator-script
$ python generator-script/main.py src/nas/generated
$ cargo test
```

If you have a directory of pcaps you'd like to generate tests from, pass its path as another argument to `generator-script/main.py`:

```
$ python generator-script/main.py src/nas/generated path/to/pcaps
```

## How this works

Parsers are rarely easy to understand, and parser generators even less so. So here's a high-level description of what our parser and its generator look like, and some details of how they work.

### The NAS parser (rust)

Our generated parser is based entirely on the [deku](https://docs.rs/deku/latest/deku/index.html) library, which is a declarative way of writing binary parsers. By declarative, we mean that our code _declares_ the structure of something rather than manually writing the process of how to parse it.

To make this more concrete, let's look at an example parser for the NAS `EMMAttachReject` message:

```rust
#[derive(DekuRead, Debug, Serialize, Clone)]
pub struct EMMAttachReject {
    #[deku(bytes = 1)] pub emm_cause: Type3V<EMMCauseEMMCause>,
    #[deku(ctx = "Tag(120), NeedsByteSize")] pub esm_container: Type6TLVE<Layer3Buffer>,
    #[deku(ctx = "Tag(95)")] pub t_3346: Type4TLV<GPRSTimer>,
    #[deku(ctx = "Tag(22)")] pub t_3402: Type4TLV<GPRSTimer>,
    #[deku(ctx = "Tag(10)")] pub ext_emm_cause: Type1TV<ExtEMMCause>,
}
```

By deriving `DekuRead`, we're saying that to parse this struct, the deku library just needs to recursively descend into each of its members and use their implementations of `DekuRead`, which is provided for many existing Rust primitive types. For custom types, such as the the timer value `Type4TLV<GPRSTimer>`, we either need to manually implement `DekuRead` or automatically derive it. For now let's ignore that wrapper type `Type4TLV` and the `#[deku(ctx = "...")]` bits and just check out the definition of `GPRSTimer`:

```rust
#[derive(DekuRead, Debug, Serialize, Clone)]
pub struct GPRSTimer {
    pub unit: GPRSTimerUnit,
    #[deku(bits = 5)] pub value: u8,
}
```

Again we're deriving `DekuRead`, so `GPRSTimer`'s implementation will consist of reading a `GPRSTimerUnit` (another custom type) and a `u8`. But note that we're decorating that `u8` with a `#[deku(bits = 5)]` macro attribute. This tells deku that reading this 1 byte unsigned integer only requires 5 bits. That's all deku needs to generate the parser for `value`. Finally, let's check out this `GPRSTimerUnit` custom type:

```rust
#[derive(DekuRead, Debug, Serialize, Clone, PartialEq)]
#[deku(id_type = "u8", bits = 3)]
pub enum GPRSTimerUnit {
    #[deku(id_pat = "0")] TwoSec,
    #[deku(id_pat = "1")] OneMin,
    #[deku(id_pat = "2")] SixMin,
    #[deku(id_pat = "7")] TimerDeactivated,
    #[deku(id_pat = "_")] Other,
}
```

Here we're declaring an enum, and telling deku that to read it, it just has to read a 3-bit-long `u8` and pattern-match its value against the provided enum variants. So ultimately, to read the whole `GPRSTimer` value, deku's generated parser will read a 3-bit-long `u8` that's pattern matched against this enum's variants, and then a 5-bit-long `u8` which is stored directly in `value`.

This pattern of declaring structs which either contain other structs, primitive rust types, or custom enums comprises nearly all of our generated NAS parser. But importantly, we still haven't talked about those toplevel `Type3V` and `Type4TLV` types or their weird attributes (e.g. `#[deku(ctx = "Tag(10))]`)

NAS messages as defined by 3GPP consist of a series of container types which are assigned numbers like `Type1`, `Type2`, up to `Type6`. And each of these container types can have optionally have tags (`T`), lengths (`L`), and values (`V`). Hence, in `src/nas/layer3.rs`, we manually define how the `DekuRead` trait should work for each of these containers. Hopefully the code for these implementations is mostly self explanatory, and it's largely just different variants of:

  1. read the tag, see if it matches our expected tag
  2. read the length, then read that length worth of bytes
  3. parse the bytes as the contained type

Passing in the expected tag for a container type is done via the [deku context attribute](https://docs.rs/deku/latest/deku/attributes/index.html#ctx). So `#[deku(ctx = "Tag(22)")] pub t_3402: Type4TLV<GPRSTimer>,` tells the `Type4TLV` container's parser that it should expect to read a tag whose value is `22`. If it reads some other tag, that means that this value isn't present, and the inner value will be `None`.

Finally, we have an unfortunate hack for container types which have variable size, and that's the `NeedsByteSize` attribute. This is described in more detail in `src/nas/layer3.rs`, but basically is needed whenever we have a variable-length byte array or string.

As a rule, the generated rust code all lives in `src/nas/generated` and has a comment at the top declaring as much, while the rest of the rust code is hand-written.

### The parser generator (python)

To generate the rust code we just analyzed, our python generator script traverses some internal implementation details within pycrate's representations of those messages.

Let's look at pycrate's implementation of the same NAS message from above. Its class definition is pretty simple:

```python
# in pycrate_mobile/TS24301_EMM.py`
class EMMAttachReject(Layer3E):
    _GEN = (
        EMMHeader(val={'Type':68}),
        Type3V('EMMCause', val={'V':b'\x11'}, bl={'V':8}, IE=EMMCause()),
        Type6TLVE('ESMContainer', val={'T':0x78, 'V':b'\0\0\0'}),
        Type4TLV('T3346', val={'T':0x5F, 'V':b'\0'}, IE=GPRSTimer()),
        Type4TLV('T3402', val={'T':0x16, 'V':b'\0'}, IE=GPRSTimer()),
        Type1TV('ExtEMMCause', val={'T':0xA, 'V':0}, IE=ExtEMMCause())
        )
```

We can see the same values from the rust code: a `Type3V` cause, a `Type6TLVE` container value, and so on.

Let's peek into the python version of `GPRSTimer`:

```python
# pycrate_mobile/TS24008_IE.py
class GPRSTimer(Envelope):
    _GEN = (
        Uint('Unit', bl=3, dic=_GPRSTimerUnit_dict),
        Uint('Value', bl=5)
        )
```

Here we can see something very similar to our rust code: a `unit` variable whose bitlength (`bl`) is 3, and a value whose bitlength is 5. Because the `unit`'s integer also declares a dictionary (`dic`) value, pycrate knows it should be one of the familiar values declared in that dictionary:

```python
# pycrate_mobile/TS24008_IE.py
_MMTimerUnit_dict = {
    0 : '2 sec',
    1 : '1 min',
    2 : '6 min',
    7 : 'timer deactivated'
    }
```

By traversing these class definitions, we can begin to programatically build a tree structure of what the entire NAS message parser should look like. This is indeed what the parser generator does, and it does so specifically by distinguishing between two core pycrate types: `pycrate_core.elt.Envelope` and `pycrate_core.elt.Atom`.

`EMMAttachReject` is a subclass of `pycrate_mobile.TS24007.Layer3E`, which is ultimately a subclass of `Envelope`. `Envelope`s are used to represent parsable containers of other values, which themselves may also be subclasses of `Envelope`s or `Atom`s. `Atom`s repesent self-contained values (usually a number or buffer). And as you might guess, the `_GEN` tuple is how an `Envelope` knows what types of values it'll contain.

(In practice, we actually use an instance variable of an `Envelope` called `_content` which is an array of a message's instantiated values, but it's composed of the same elements listed in `_GEN`)

Other pycrate innards like the aforementioned bitlength (`bl`) value or an enum's dictionary of variants (`dic`) are further used to flesh out what our resulting rust code looks like.

At this point it's worth noting that `_content` and many of the other values we'll be referencing are internal/private values on pycrate objects, that means the generator is inherently brittle and reliant on a specific version of pycrate. Unless pycrate decides to turn these into public API (which I don't think they're interested in), this is a risk we'll have to take.

There's obviously a lot more detail to how this works, as well as how rust test generation works, but these nitty gritty bits are documented fairly thoroughly in `generator-script`.
