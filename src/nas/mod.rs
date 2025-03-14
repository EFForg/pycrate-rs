use deku::prelude::*;
use deku::ctx::BitSize;
use emm::{parse_emm_nas, EMMType};
use esm::{parse_esm_nas, ESMType};
use std::io::Cursor;
use thiserror::Error;
use serde::Serialize;

pub mod layer3;
pub mod emm;
pub mod esm;
pub mod generated;

mod test_utils;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Cannot decode encrypted NAS messages")]
    EncryptedNASMessage,
    #[error("Unsupported NAS protocol {0:?}")]
    UnsupportedNASProtocol(ProtocolDiscriminator),
    #[error("Failed to parse message")]
    Deku(#[from] DekuError),
}

#[derive(Clone, Debug, Serialize)]
pub enum NASMessage {
    EMMMessage(emm::EMMMessage),
    ESMMessage(esm::ESMMessage),
}

impl NASMessage {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);
        let mut reader = Reader::new(&mut cursor);
        let sec_hdr_or_bearer_id = u8::from_reader_with_ctx(&mut reader, BitSize(4))?;
        match ProtocolDiscriminator::from_reader_with_ctx(&mut reader, ())? {
            ProtocolDiscriminator::EMM => {
                if sec_hdr_or_bearer_id != SecHdrType::NoSecurity.deku_id()? {
                    return Err(ParseError::EncryptedNASMessage);
                }
                let emm_type = EMMType::from_reader_with_ctx(&mut reader, ())?;
                Ok(NASMessage::EMMMessage(parse_emm_nas(emm_type, reader)?))
            },
            ProtocolDiscriminator::ESM => {
                let _pti = u8::from_reader_with_ctx(&mut reader, ())?;
                let esm_type = ESMType::from_reader_with_ctx(&mut reader, ())?;
                Ok(NASMessage::ESMMessage(parse_esm_nas(esm_type, reader)?))
            }
            p => Err(ParseError::UnsupportedNASProtocol(p)),
        }
    }
}

#[derive(DekuRead, DekuWrite, Debug)]
pub struct NASHeader {
    pub sec_hdr: SecHdrType,
    pub protocol_discriminator: ProtocolDiscriminator,
}

#[derive(DekuRead, DekuWrite, Debug)]
#[deku(id_type = "u8", bits = 4)]
pub enum SecHdrType {
    #[deku(id = 0)] NoSecurity,
    #[deku(id = 1)] IntegrityProtected,
    #[deku(id = 2)] IntegrityProtectedAndCiphered,
    #[deku(id = 3)] IntegrityProtectedNewEPS,
    #[deku(id = 4)] IntegrityProtectedAndCipheredNewEPS,
    #[deku(id = 12)] SecurityHeaderForServiceRequest,
}

#[derive(DekuRead, DekuWrite, Debug)]
#[deku(id_type = "u8", bits = 4)]
pub enum ProtocolDiscriminator {
    #[deku(id = 0)] GCC,
    #[deku(id = 1)] BCC,
    #[deku(id = 2)] ESM,
    #[deku(id = 3)] CC,
    #[deku(id = 4)] GTTP,
    #[deku(id = 5)] MM,
    #[deku(id = 6)] RRM,
    #[deku(id = 7)] EMM,
    #[deku(id = 8)] GMM,
    #[deku(id = 9)] SMS,
    #[deku(id = 10)] SM,
    #[deku(id = 11)] SS,
    #[deku(id = 12)] LCS,
    #[deku(id = 14)] ExtendedProtDisc,
    #[deku(id = 15)] Testing,
    #[deku(id = 46)] FiveGSM,
    #[deku(id = 126)] FiveGMM,
}
