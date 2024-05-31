use core::marker::PhantomData;

use crate::de::{r#as::BitUnpackAs, BitReader, BitReaderExt};

use super::Same;

pub struct DefaultOnNone<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> BitUnpackAs<T> for DefaultOnNone<As>
where
    T: Default,
    As: BitUnpackAs<T>,
{
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        reader
            .unpack_as::<_, Option<As>>()
            .map(Option::unwrap_or_default)
    }
}
