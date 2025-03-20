use std::io::{Cursor, Read, Seek};
use std::marker::PhantomData;

use deku::ctx::{BitSize, ByteSize, Endian};
use deku::prelude::*;
use serde::Serialize;

// A kind of marker type that exists as kind of a hack. This is needed because
// the following types, there's a lot of impls that look like `impl DekuRead<A>
// for Type<B> where B: DekuRead<C>`, and frequently that impl needs to be
// different depending on whether C includes the `ByteSize` context. But Rust
// doesn't allow separate `impl DekuRead<A> for Type<B> where B: DekuRead<C>`
// for different values of C (this is a feature known as "trait
// specialization"). So, we shoehorn this empty type into `A` whenever `C`
// differs based on the existence of `ByteSize`.
pub struct NeedsByteSize;

#[derive(Serialize, DekuRead, Debug, Clone)]
#[deku(ctx = "ByteSize(byte_size): ByteSize")]
pub struct Layer3Buffer {
    #[deku(count = "byte_size")] pub buf: Vec<u8>,
}

impl PartialEq<Vec<u8>> for Layer3Buffer {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.buf.eq(other)
    }
}

// IE formats described in Sec 11.2.1.1 of 3GPP TS 24.007

#[derive(Serialize, Debug, Clone)]
pub struct Type1TV<T> {
    tag: u8,
    v: u8,
    pub inner: Option<T>,
}

impl<'a, T> DekuReader<'a, Tag> for Type1TV<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        tag: Tag
    ) -> Result<Self, DekuError> {
        if reader.end() {
            return Ok(Self { tag: tag.into(), v: 0, inner: None });
        }
        let t = u8::from_reader_with_ctx(reader, BitSize(4))?;
        let v = u8::from_reader_with_ctx(reader, BitSize(4))?;
        if t != tag.0 {
            reader.seek_relative(-1)
                .map_err(|err| DekuError::Io(err.kind()))?;
            return Ok(Self { tag: tag.into(), v: 0, inner: None });
        }
        let mut cursor = Cursor::new([v]);
        let mut inner_reader = Reader::new(&mut cursor);
        inner_reader.skip_bits(4)?;
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ())?);
        Ok(Self { tag: tag.into(), v, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type1V<T> {
    pub v: u8,
    pub inner: T,
}

impl<'a, T> DekuReader<'a> for Type1V<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        _: ()
    ) -> Result<Self, DekuError> {
        let v = u8::from_reader_with_ctx(reader, BitSize(4))?;
        let mut cursor = Cursor::new([v]);
        let mut inner_reader = Reader::new(&mut cursor);
        inner_reader.skip_bits(4)?;
        let inner = T::from_reader_with_ctx(&mut inner_reader, ())?;
        Ok(Type1V { v, inner })
    }
}

// maybe unused?
#[derive(Serialize, DekuRead, DekuWrite, Debug, Clone)]
pub struct Type2<T> {
    pub tag: u8,
    #[deku(skip)] pub _phantom: PhantomData<T>
}

#[derive(Serialize, Debug, Clone)]
pub struct Type3V<T> {
    pub inner: T,
}

impl<'a, T> DekuReader<'a, ByteSize> for Type3V<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        ByteSize(byte_size): ByteSize
    ) -> Result<Self, DekuError> {
        let buf = read_bytes_from_reader(reader, byte_size)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ())?;
        Ok(Type3V { inner })
    }
}

impl<'a, T> DekuReader<'a, (ByteSize, NeedsByteSize)> for Type3V<T> where T: DekuReader<'a, ByteSize> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        (ByteSize(byte_size), _): (ByteSize, NeedsByteSize)
    ) -> Result<Self, DekuError> {
        let buf = read_bytes_from_reader(reader, byte_size)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ByteSize(byte_size))?;
        Ok(Type3V { inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type3TV<T> {
    tag: u8,
    pub inner: Option<T>,
}

impl<'a, T> DekuReader<'a, (ByteSize, Tag)> for Type3TV<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        (ByteSize(byte_size), tag): (ByteSize, Tag)
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), inner: None });
        }
        let buf = read_bytes_from_reader(reader, byte_size)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ())?);
        Ok(Self { tag: tag.into(), inner })
    }
}

impl<'a, T> DekuReader<'a, (ByteSize, Tag, NeedsByteSize)> for Type3TV<T> where T: DekuReader<'a, ByteSize> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        (ByteSize(byte_size), tag, _): (ByteSize, Tag, NeedsByteSize)
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), inner: None });
        }
        let buf = read_bytes_from_reader(reader, byte_size)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ByteSize(byte_size))?);
        Ok(Self { tag: tag.into(), inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type4LV<T> {
    pub length: u8,
    pub inner: T,
}

impl<'a, T> DekuReader<'a, NeedsByteSize> for Type4LV<T> where T: DekuReader<'a, ByteSize> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        _: NeedsByteSize
    ) -> Result<Self, DekuError> {
        let length = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ByteSize(length as usize))?;
        Ok(Self { length, inner })
    }
}

impl<'a, T> DekuReader<'a> for Type4LV<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        _: ()
    ) -> Result<Self, DekuError> {
        let length = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ())?;
        Ok(Self { length, inner })
    }
}

// checks whether the next byte matches the given tag. if it does, the reader will remain
// advanced, and if not, it'll be rewound by 1 byte.
fn check_tag<R: Read+Seek>(reader: &mut Reader<R>, bit_size: BitSize, Tag(tag): Tag) -> Result<bool, DekuError> {
    if reader.end() {
        return Ok(false);
    }
    let read_tag = u8::from_reader_with_ctx(reader, bit_size)?;
    if read_tag == tag {
        Ok(true)
    } else {
        // the tag didn't match, so rewind and pretend this never happened
        // reader.seek_last_read()
        reader.seek_relative(-1)
            .map_err(|err| DekuError::Io(err.kind()))?;
        Ok(false)
    }
}

// workaround for https://github.com/sharksforarms/deku/issues/527, which
// results in a possible panic if we read more than 16 bytes at a time
fn read_bytes_from_reader<R: Read+Seek>(reader: &mut Reader<R>, mut amt: usize) -> Result<Vec<u8>, DekuError> {
    let mut result = Vec::with_capacity(amt);
    while amt > 0 {
        let amt_to_read: usize;
        if amt <= 16 {
            amt_to_read = amt;
            amt = 0;
        } else {
            amt -= 16;
            amt_to_read = 16;
        };
        let mut buf = vec![0_u8; amt_to_read];
        reader.read_bytes(amt_to_read, &mut buf)?;
        result.extend(buf);
    }
    Ok(result)
}

#[derive(Copy, Clone, Debug)]
pub struct Tag(pub u8);

impl From<Tag> for u8 {
    fn from(t: Tag) -> u8 {
        t.0
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Type4TLV<T> {
    tag: u8,
    length: u8,
    pub inner: Option<T>,
}

impl<'a, T> DekuReader<'a, (Tag, NeedsByteSize)> for Type4TLV<T> where T: DekuReader<'a, ByteSize> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        (tag, _): (Tag, NeedsByteSize)
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), length: 0, inner: None });
        }
        let length = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ByteSize(length as usize))?);
        Ok(Self { tag: tag.into(), length, inner })
    }
}

impl<'a, T> DekuReader<'a, Tag> for Type4TLV<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        tag: Tag
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), length: 0, inner: None });
        }
        let length = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ())?);
        Ok(Self { tag: tag.into(), length, inner })
    }
}

impl<'a, T> DekuReader<'a, (Endian, Tag)> for Type4TLV<T> where T: DekuReader<'a, Endian> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        (endian, tag): (Endian, Tag)
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), length: 0, inner: None });
        }
        let length = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, endian)?);
        Ok(Self { tag: tag.into(), length, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type6LVE<T> {
    pub length: u16,
    pub inner: T,
}

impl<'a, T> DekuReader<'a> for Type6LVE<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        _: ()
    ) -> Result<Self, DekuError> {
        let length = u16::from_reader_with_ctx(reader, Endian::Big)?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ())?;
        Ok(Self { length, inner })
    }
}

impl<'a, T> DekuReader<'a, NeedsByteSize> for Type6LVE<T> where T: DekuReader<'a, ByteSize> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        _: NeedsByteSize
    ) -> Result<Self, DekuError> {
        let length = u16::from_reader_with_ctx(reader, Endian::Big)?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ByteSize(length as usize))?;
        Ok(Self { length, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type6TLVE<T> {
    tag: u8,
    length: u16,
    pub inner: Option<T>,
}

impl<'a, T> DekuReader<'a, Tag> for Type6TLVE<T> where T: DekuReader<'a> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        tag: Tag
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), length: 0, inner: None });
        }
        let length = u16::from_reader_with_ctx(reader, Endian::Big)?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ())?);
        Ok(Self { tag: tag.into(), length, inner })
    }
}

impl<'a, T> DekuReader<'a, (Tag, NeedsByteSize)> for Type6TLVE<T> where T: DekuReader<'a, ByteSize> {
    fn from_reader_with_ctx<R: Read+Seek>(
        reader: &mut Reader<R>,
        (tag, _): (Tag, NeedsByteSize),
    ) -> Result<Self, DekuError> {
        if !check_tag(reader, BitSize(8), tag)? {
            return Ok(Self { tag: tag.into(), length: 0, inner: None });
        }
        let length = u16::from_reader_with_ctx(reader, Endian::Big)?;
        let buf = read_bytes_from_reader(reader, length as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Some(T::from_reader_with_ctx(&mut inner_reader, ByteSize(length as usize))?);
        Ok(Self { tag: tag.into(), length, inner })
    }
}
