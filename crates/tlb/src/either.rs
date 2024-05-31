use crate::{
    bits::{
        de::{r#as::UnpackAsWrap, BitReaderExt},
        ser::{r#as::PackAsWrap, BitWriterExt},
        Either,
    },
    de::{r#as::CellDeserializeAs, CellDeserialize, CellParser, CellParserError},
    r#as::Same,
    ser::{r#as::CellSerializeAs, CellBuilder, CellBuilderError, CellSerialize},
    ResultExt, StringError,
};

impl<L, R> CellSerialize for Either<L, R>
where
    L: CellSerialize,
    R: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Left(l) => builder
                .pack(false)
                .context("tag")?
                .store(l)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true)
                .context("tag")?
                .store(r)
                .context("right")?,
        };
        Ok(())
    }
}

impl<'de, Left, Right> CellDeserialize<'de> for Either<Left, Right>
where
    Left: CellDeserialize<'de>,
    Right: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        match parser.unpack().context("tag")? {
            false => parser.parse().map(Either::Left).context("left"),
            true => parser.parse().map(Either::Right).context("right"),
        }
    }
}

impl<Left, Right, AsLeft, AsRight> CellSerializeAs<Either<Left, Right>> for Either<AsLeft, AsRight>
where
    AsLeft: CellSerializeAs<Left>,
    AsRight: CellSerializeAs<Right>,
{
    #[inline]
    fn store_as(
        source: &Either<Left, Right>,
        builder: &mut CellBuilder,
    ) -> Result<(), CellBuilderError> {
        source
            .as_ref()
            .map_either(
                PackAsWrap::<Left, AsLeft>::new,
                PackAsWrap::<Right, AsRight>::new,
            )
            .store(builder)
    }
}

impl<'de, Left, Right, AsLeft, AsRight> CellDeserializeAs<'de, Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: CellDeserializeAs<'de, Left>,
    AsRight: CellDeserializeAs<'de, Right>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Either<Left, Right>, CellParserError<'de>> {
        Ok(
            Either::<UnpackAsWrap<Left, AsLeft>, UnpackAsWrap<Right, AsRight>>::parse(parser)?
                .map_either(UnpackAsWrap::into_inner, UnpackAsWrap::into_inner),
        )
    }
}

impl<T, As> CellSerializeAs<Option<T>> for Either<(), As>
where
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match source.as_ref() {
            None => Either::Left(()),
            Some(v) => Either::Right(PackAsWrap::<T, As>::new(v)),
        }
        .store(builder)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Option<T>> for Either<(), As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Option<T>, StringError> {
        Ok(Either::<(), UnpackAsWrap<T, As>>::parse(parser)?
            .map_right(UnpackAsWrap::into_inner)
            .right())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> CellSerialize for Option<T>
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_as::<_, Either<(), Same>>(self.as_ref())?;
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<'de, T> CellDeserialize<'de> for Option<T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, StringError> {
        parser.parse_as::<_, Either<(), Same>>()
    }
}
