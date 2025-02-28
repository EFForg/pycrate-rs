use std::io::{Cursor, Read, Seek};
use std::marker::PhantomData;

use deku::ctx::{BitSize, ByteSize, Endian};
use deku::prelude::*;
use serde::Serialize;

// Used for container types which don't have a specific inner value
#[derive(DekuRead, DekuWrite, Debug)]
pub struct NoneType;

#[derive(Serialize, DekuRead, Debug, Clone)]
#[deku(ctx = "ByteSize(byte_size): ByteSize")]
pub struct Layer3Buffer {
    #[deku(count = "byte_size")] pub buf: Vec<u8>,
}

// IE formats described in Sec 11.2.1.1 of 3GPP TS 24.007

#[derive(Serialize, Debug, Clone)]
pub struct Type1TV<T> {
    pub t: u8,
    pub v: u8,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type1TV<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>,ctx: Ctx) -> Result<Self,DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, BitSize(4))?;
        let v = u8::from_reader_with_ctx(reader, BitSize(4))?;
        let mut cursor = Cursor::new([v]);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Type1TV { t, v, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type1V<T> {
    pub v: u8,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type1V<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let v = u8::from_reader_with_ctx(reader, BitSize(4))?;
        let mut cursor = Cursor::new([v]);
        let mut inner_reader = Reader::new(&mut cursor);
        inner_reader.skip_bits(4)?;
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Type1V { v, inner })
    }
}

#[derive(Serialize, DekuRead, DekuWrite, Debug, Clone)]
pub struct Type2<T> {
    pub t: u8,
    #[deku(skip)] pub _phantom: PhantomData<T>
}

#[derive(Serialize, Debug, Clone)]
pub struct Type3V<T> {
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type3V<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let inner = T::from_reader_with_ctx(reader, ctx)?;
        Ok(Type3V { inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type3TV<T> {
    pub t: u8,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type3TV<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, ())?;
        let inner = T::from_reader_with_ctx(reader, ctx)?;
        Ok(Self { t, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type4LV<T> {
    pub l: u8,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type4LV<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let l = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, l as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { l, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type4TLV<T> {
    pub t: u8,
    pub l: u8,
    pub inner: T,
}

// workaround for https://github.com/sharksforarms/deku/issues/527, which
// results in a possible panic if we read more than 16 bytes at a time
fn read_bytes_from_reader<R: Read+Seek>(reader: &mut Reader<R>, mut amt: usize) -> Result<Vec<u8>, DekuError> {
    let mut result = vec![0_u8; amt];
    while amt > 0 {
        let mut buf = vec![0_u8; 16];
        let amt_to_read: usize;
        if amt <= 16 {
            amt_to_read = amt;
            amt = 0;
        } else {
            amt -= 16;
            amt_to_read = 16;
        };
        reader.read_bytes(amt_to_read, &mut buf)?;
        result.extend(buf);
    }
    Ok(result)
}

impl<'a> DekuReader<'a> for Type4TLV<Layer3Buffer> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, _: ()) -> Result<Self, DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, ())?;
        let l = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, l as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = Layer3Buffer::from_reader_with_ctx(&mut inner_reader, ByteSize(l as usize))?;
        Ok(Self { t, l, inner })
    }
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type4TLV<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, ())?;
        let l = u8::from_reader_with_ctx(reader, ())?;
        let buf = read_bytes_from_reader(reader, l as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { t, l, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type6LVE<T> {
    pub l: u16,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type6LVE<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let l = u16::from_reader_with_ctx(reader, Endian::Big)?;
        let buf = read_bytes_from_reader(reader, l as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { l, inner })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Type6TLVE<T> {
    pub t: u8,
    pub l: u16,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type6TLVE<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, ())?;
        let l = u16::from_reader_with_ctx(reader, Endian::Big)?;
        let buf = read_bytes_from_reader(reader, l as usize)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { t, l, inner })
    }
}
