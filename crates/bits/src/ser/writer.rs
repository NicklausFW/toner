use ::bitvec::{order::Msb0, slice::BitSlice, store::BitStore, vec::BitVec, view::AsBits};
use impl_tools::autoimpl;

use crate::{
    adapters::{BitCounter, Tee},
    Error, ResultExt, StringError,
};

use super::{
    args::{r#as::BitPackAsWithArgs, BitPackWithArgs},
    r#as::BitPackAs,
    BitPack,
};

#[autoimpl(for <W: trait + ?Sized> &mut W, Box<W>)]
pub trait BitWriter {
    type Error: Error;

    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error>;

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        for bit in bits {
            self.write_bit(*bit)?;
        }
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        for _ in 0..n {
            self.write_bit(bit)?;
        }
        Ok(())
    }
}

pub trait BitWriterExt: BitWriter {
    #[inline]
    fn with_bit(&mut self, bit: bool) -> Result<&mut Self, Self::Error> {
        self.write_bit(bit)?;
        Ok(self)
    }

    #[inline]
    fn with_bits(
        &mut self,
        bits: impl AsRef<BitSlice<u8, Msb0>>,
    ) -> Result<&mut Self, Self::Error> {
        self.write_bitslice(bits.as_ref())?;
        Ok(self)
    }

    #[inline]
    fn with_repeat_bit(&mut self, n: usize, bit: bool) -> Result<&mut Self, Self::Error> {
        self.repeat_bit(n, bit)?;
        Ok(self)
    }

    #[inline]
    fn with_bytes(&mut self, bytes: impl AsRef<[u8]>) -> Result<&mut Self, Self::Error> {
        self.with_bits(bytes.as_bits::<Msb0>())?;
        Ok(self)
    }

    #[inline]
    fn pack<T>(&mut self, value: T) -> Result<&mut Self, Self::Error>
    where
        T: BitPack,
    {
        value.pack::<&mut Self>(self)?;
        Ok(self)
    }

    #[inline]
    fn pack_with<T>(&mut self, value: T, args: T::Args) -> Result<&mut Self, Self::Error>
    where
        T: BitPackWithArgs,
    {
        value.pack_with::<&mut Self>(self, args)?;
        Ok(self)
    }

    #[inline]
    fn pack_many<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, Self::Error>
    where
        T: BitPack,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn pack_many_with<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: T::Args,
    ) -> Result<&mut Self, Self::Error>
    where
        T: BitPackWithArgs,
        T::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack_with(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn pack_as<T, As>(&mut self, value: T) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAs<T> + ?Sized,
    {
        As::pack_as::<&mut Self>(&value, self)?;
        Ok(self)
    }

    #[inline]
    fn pack_as_with<T, As>(&mut self, value: T, args: As::Args) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAsWithArgs<T> + ?Sized,
    {
        As::pack_as_with::<&mut Self>(&value, self, args)?;
        Ok(self)
    }

    #[inline]
    fn pack_many_as<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAs<T> + ?Sized,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack_as::<_, As>(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn pack_many_as_with<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: As::Args,
    ) -> Result<&mut Self, Self::Error>
    where
        As: BitPackAsWithArgs<T> + ?Sized,
        As::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.pack_as_with::<_, As>(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }

    #[inline]
    fn counted(self) -> BitCounter<Self>
    where
        Self: Sized,
    {
        BitCounter::new(self)
    }

    #[inline]
    fn limit(self, n: usize) -> LimitWriter<Self>
    where
        Self: Sized,
    {
        LimitWriter::new(self, n)
    }

    #[inline]
    fn tee<W>(self, writer: W) -> Tee<Self, W>
    where
        Self: Sized,
        W: BitWriter,
    {
        Tee {
            inner: self,
            writer,
        }
    }
}

impl<T> BitWriterExt for T where T: BitWriter {}

impl<W> BitWriter for BitCounter<W>
where
    W: BitWriter,
{
    type Error = W::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.inner.write_bit(bit)?;
        self.counter += 1;
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.write_bitslice(bits)?;
        self.counter += bits.len();
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.inner.repeat_bit(n, bit)?;
        self.counter += n;
        Ok(())
    }
}

#[autoimpl(Deref using self.inner)]
pub struct LimitWriter<W> {
    inner: BitCounter<W>,
    limit: usize,
}

impl<W> LimitWriter<W>
where
    W: BitWriter,
{
    #[inline]
    pub const fn new(writer: W, limit: usize) -> Self {
        Self {
            inner: BitCounter::new(writer),
            limit,
        }
    }

    #[inline]
    fn ensure_more(&self, n: usize) -> Result<(), W::Error> {
        if self.bit_count() + n > self.limit {
            return Err(Error::custom("max bits limit reached"));
        }
        Ok(())
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.inner.into_inner()
    }
}

impl<W> BitWriter for LimitWriter<W>
where
    W: BitWriter,
{
    type Error = W::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.ensure_more(1)?;
        self.inner.write_bit(bit)
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.ensure_more(bits.len())?;
        self.inner.write_bitslice(bits)
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.ensure_more(n)?;
        self.inner.repeat_bit(n, bit)
    }
}

impl<T, W> BitWriter for Tee<T, W>
where
    T: BitWriter,
    W: BitWriter,
{
    type Error = T::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.inner.write_bit(bit)?;
        self.writer
            .write_bit(bit)
            .map_err(<T::Error>::custom)
            .context("writer")
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.inner.write_bitslice(bits)?;
        self.writer
            .write_bitslice(bits)
            .map_err(<T::Error>::custom)
            .context("writer")
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.inner.repeat_bit(n, bit)?;
        self.writer
            .repeat_bit(n, bit)
            .map_err(<T::Error>::custom)
            .context("writer")
    }
}

impl<S> BitWriter for BitVec<S, Msb0>
where
    S: BitStore,
{
    type Error = StringError;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.push(bit);
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.extend_from_bitslice(bits);
        Ok(())
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.resize(self.len() + n, bit);
        Ok(())
    }
}
