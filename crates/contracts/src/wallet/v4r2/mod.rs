use std::sync::Arc;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use nacl::sign::PUBLIC_KEY_LENGTH;
use tlb::{
    bits::{de::BitReaderExt, integer::ConstBit, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerialize},
    Cell,
};
use tlb_ton::{BagOfCells, UnixTimestamp};

use super::{WalletOpSendMessage, WalletVersion};

lazy_static! {
    static ref WALLET_V4R2_CODE_CELL: Arc<Cell> = {
        BagOfCells::parse_base64(include_str!("./wallet_v4r2.code"))
            .unwrap()
            .single_root()
            .expect("code BoC must be single root")
            .clone()
    };
}

pub struct V4R2;

impl WalletVersion for V4R2 {
    type Data = WalletV4R2Data;
    type MessageBody = WalletV4R2Message;

    fn code() -> Arc<Cell> {
        WALLET_V4R2_CODE_CELL.clone()
    }

    fn init_data(wallet_id: u32, pubkey: [u8; PUBLIC_KEY_LENGTH]) -> Self::Data {
        WalletV4R2Data {
            seqno: 0,
            wallet_id,
            pubkey,
        }
    }

    fn create_external_body(
        wallet_id: u32,
        expire_at: DateTime<Utc>,
        seqno: u32,
        msgs: impl IntoIterator<Item = WalletOpSendMessage>,
    ) -> Self::MessageBody {
        WalletV4R2Message {
            wallet_id,
            expire_at,
            seqno,
            op: WalletV4R2Op::Send(msgs.into_iter().collect()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletV4R2Data {
    pub seqno: u32,
    pub wallet_id: u32,
    pub pubkey: [u8; PUBLIC_KEY_LENGTH],
}

impl CellSerialize for WalletV4R2Data {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.seqno)?
            .pack(self.wallet_id)?
            .pack(self.pubkey)?
            // TODO: handle plugins dict
            .pack(false)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for WalletV4R2Data {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        let d = Self {
            seqno: parser.unpack()?,
            wallet_id: parser.unpack()?,
            pubkey: parser.unpack()?,
        };
        // TODO: plugins
        let _plugins: ConstBit<false> = parser.unpack()?;
        Ok(d)
    }
}

pub struct WalletV4R2Message {
    pub wallet_id: u32,
    pub expire_at: DateTime<Utc>,
    pub seqno: u32,
    pub op: WalletV4R2Op,
}

impl CellSerialize for WalletV4R2Message {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.wallet_id)?
            .pack_as::<_, UnixTimestamp>(self.expire_at)?
            .pack(self.seqno)?
            .store(&self.op)?;
        Ok(())
    }
}

pub enum WalletV4R2Op {
    Send(Vec<WalletOpSendMessage>),
    // TODO: add support for plugins management
}

impl CellSerialize for WalletV4R2Op {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Send(msgs) => builder.pack(0u8)?.store_many(msgs)?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tlb::bits::{de::unpack_fully, ser::pack_with};
    use tlb_ton::{BagOfCellsArgs, BoC};

    use super::*;

    #[test]
    fn check_code() {
        let packed = pack_with(
            BoC::from_root(WALLET_V4R2_CODE_CELL.clone()),
            BagOfCellsArgs {
                has_idx: false,
                has_crc32c: true,
            },
        )
        .unwrap();

        let unpacked: BoC = unpack_fully(packed).unwrap();

        let got: Cell = unpacked.single_root().unwrap().parse_fully().unwrap();
        assert_eq!(&got, WALLET_V4R2_CODE_CELL.as_ref());
    }
}
