pub mod args;
pub mod r#as;
mod writer;

pub use self::writer::*;

use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use either::Either;
use impl_tools::autoimpl;

use crate::{
    r#as::{AsBytes, Same},
    ResultExt, StringError,
};

use self::args::BitPackWithArgs;

#[autoimpl(for<S: trait + ?Sized> &S, &mut S, Box<S>, Rc<S>, Arc<S>)]
pub trait BitPack {
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter;
}

#[inline]
pub fn pack<T>(value: T) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPack,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack(&mut writer, value)?;
    Ok(writer)
}

#[inline]
pub fn pack_with<T>(value: T, args: T::Args) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPackWithArgs,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack_with(&mut writer, value, args)?;
    Ok(writer)
}

impl BitPack for () {
    #[inline]
    fn pack<W>(&self, _writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        Ok(())
    }
}

impl BitPack for bool {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.write_bit(*self).map_err(Into::into)
    }
}

impl<T> BitPack for [T]
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_many(self)?;
        Ok(())
    }
}

impl<T, const N: usize> BitPack for [T; N]
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack(writer)
    }
}

impl<T> BitPack for Vec<T>
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack(writer)
    }
}

macro_rules! impl_bit_pack_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> BitPack for ($($t,)+)
        where $(
            $t: BitPack,
        )+
        {
            #[inline]
            fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                $(self.$n.pack(&mut writer).context(concat!(".", stringify!($n)))?;)+
                Ok(())
            }
        }
    };
}
impl_bit_pack_for_tuple!(0:T0);
impl_bit_pack_for_tuple!(0:T0,1:T1);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<L, R> BitPack for Either<L, R>
where
    L: BitPack,
    R: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Left(l) => writer.pack(false).context("tag")?.pack(l).context("left")?,
            Self::Right(r) => writer.pack(true).context("tag")?.pack(r).context("right")?,
        };
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> BitPack for Option<T>
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, Either<(), Same>>(self.as_ref())?;
        Ok(())
    }
}

impl<'a> BitPack for &'a BitSlice<u8, Msb0> {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.write_bitslice(self)
    }
}

impl BitPack for BitVec<u8, Msb0> {
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_bitslice().pack(writer)
    }
}

impl BitPack for str {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, AsBytes>(self)?;
        Ok(())
    }
}
