use std::io::{Cursor, Read, Seek};

use deku::ctx::BitSize;
use deku::prelude::*;

// Used for container types which don't have a specific inner value
#[derive(DekuRead, DekuWrite, Debug)]
pub struct NoneType;

// IE formats described in Sec 11.2.1.1 of 3GPP TS 24.007

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

#[derive(Debug)]
pub struct Type1V<T> {
    pub v: u8,
    pub inner: T,
}

impl<'a, T, Ctx> DekuReader<'a, Ctx> for Type1V<T> where T: DekuReader<'a, Ctx> {
    fn from_reader_with_ctx<R: Read+Seek>(reader: &mut Reader<R>, ctx: Ctx) -> Result<Self, DekuError> where Self:Sized {
        let v = u8::from_reader_with_ctx(reader, BitSize(4))?;
        let mut cursor = Cursor::new([v]);
        let mut inner_reader = Reader::new(&mut cursor);
        let inner = T::from_reader_with_ctx(&mut inner_reader, ctx)?;
        Ok(Type1V { v, inner })
    }
}
