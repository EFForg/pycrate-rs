use std::io::{Cursor, Read, Seek};
use std::marker::PhantomData;

use deku::ctx::{BitSize, ByteSize};
use deku::prelude::*;

// Used for container types which don't have a specific inner value
#[derive(DekuRead, DekuWrite, Debug)]
pub struct NoneType;

#[derive(DekuRead, Debug, Clone)]
#[deku(ctx = "ByteSize(byte_size): ByteSize")]
pub struct Layer3Buffer {
    #[deku(count = "byte_size")] pub buf: Vec<u8>,
}

// IE formats described in Sec 11.2.1.1 of 3GPP TS 24.007

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(DekuRead, DekuWrite, Debug, Clone)]
pub struct Type2<T> {
    pub t: u8,
    #[deku(skip)] pub _phantom: PhantomData<T>
}

#[derive(Debug, Clone)]
pub struct Type3V<T> {
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type3V<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let inner = T::from_reader_with_ctx(reader, ctx)?;
        Ok(Type3V { inner })
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Type4LV<T> {
    pub l: u8,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type4LV<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let l = u8::from_reader_with_ctx(reader, ())?;
        let mut buf = vec![0_u8; l as usize];
        reader.read_bytes(l as usize, &mut buf)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { l, inner })
    }
}

#[derive(Debug, Clone)]
pub struct Type4TLV<T> {
    pub t: u8,
    pub l: u8,
    pub inner: T,
}

impl<'a> DekuReader<'a> for Type4TLV<Layer3Buffer> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, _: ()) -> Result<Self, DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, ())?;
        let l = u8::from_reader_with_ctx(reader, ())?;
        let mut buf = vec![0_u8; l as usize];
        reader.read_bytes(l as usize, &mut buf)?;
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
        let mut buf = vec![0_u8; l as usize];
        reader.read_bytes(l as usize, &mut buf)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { t, l, inner })
    }
}

#[derive(Debug, Clone)]
pub struct Type6LVE<T> {
    pub l: u16,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type6LVE<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let l = u16::from_reader_with_ctx(reader, ())?;
        let mut buf = vec![0_u8; l as usize];
        reader.read_bytes(l as usize, &mut buf)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { l, inner })
    }
}

#[derive(Debug, Clone)]
pub struct Type6TLVE<T> {
    pub t: u8,
    pub l: u16,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type6TLVE<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let t = u8::from_reader_with_ctx(reader, ())?;
        let l = u16::from_reader_with_ctx(reader, ())?;
        let mut buf = vec![0_u8; l as usize];
        reader.read_bytes(l as usize, &mut buf)?;
        let mut cursor = Cursor::new(buf);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Self { t, l, inner })
    }
}
