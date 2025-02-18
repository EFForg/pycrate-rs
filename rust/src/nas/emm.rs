use deku::prelude::*;

use super::generated::*;

#[derive(DekuRead, DekuWrite, Debug)]
struct EMMHeader {
    pub sec_hdr: SecHdrType,
    pub protocol_discriminator: ProtocolDiscriminator,
    pub emm_type: EMMType,
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
#[deku(id_type = "u8")]
pub enum EMMType {
    // attach / detach
    #[deku(id = 65)] AttachRequest,
    #[deku(id = 66)] AttachAccept,
    #[deku(id = 67)] AttachComplete,
    #[deku(id = 68)] AttachReject,
    #[deku(id = 69)] DetachRequest,
    #[deku(id = 70)] DetachAccept,
    // TAU
    #[deku(id = 72)] TrackingAreaUpdateRequest,
    #[deku(id = 73)] TrackingAreaUpdateAccept,
    #[deku(id = 74)] TrackingAreaUpdateComplete,
    #[deku(id = 75)] TrackingAreaUpdateReject,
    // serv request
    #[deku(id = 76)] ExtendedServiceRequest,
    #[deku(id = 77)] ControlPlaneServiceRequest,
    #[deku(id = 78)] ServiceReject,
    #[deku(id = 79)] ServiceAccept,
    // identification / authentication
    #[deku(id = 80)] GUTIReallocationCommand,
    #[deku(id = 81)] GUTIReallocationComplete,
    #[deku(id = 82)] AuthenticationRequest,
    #[deku(id = 83)] AuthenticationResponse,
    #[deku(id = 84)] AuthenticationReject,
    #[deku(id = 92)] AuthenticationFailure,
    #[deku(id = 85)] IdentityRequest,
    #[deku(id = 86)] IdentityResponse,
    #[deku(id = 93)] SecurityModeCommand,
    #[deku(id = 94)] SecurityModeComplete,
    #[deku(id = 95)] SecurityModeReject,
    // misc
    #[deku(id = 96)] EMMStatus,
    #[deku(id = 97)] EMMInformation,
    #[deku(id = 98)] DownlinkNASTransport,
    #[deku(id = 99)] UplinkNASTransport,
    #[deku(id = 100)] CSServiceNotification,
    #[deku(id = 104)] DownlinkGenericNASTransport,
    #[deku(id = 105)] UplinkGenericNASTransport,
}

#[derive(Debug, Clone)]
pub enum NASLTEMessage {
    EMMAttachRequest(emmattachrequest::EMMAttachRequest),
    EMMAttachAccept(emmattachaccept::EMMAttachAccept),
    EMMAttachComplete(emmattachcomplete::EMMAttachComplete),
    EMMAttachReject(emmattachreject::EMMAttachReject),
    // EMMDetachRequestMT(emmdetachrequestmt::EMMDetachRequestMT),
    EMMDetachRequestMO(emmdetachrequestmo::EMMDetachRequestMO),
    EMMDetachAccept(emmdetachaccept::EMMDetachAccept),
    EMMTrackingAreaUpdateRequest(emmtrackingareaupdaterequest::EMMTrackingAreaUpdateRequest),
    EMMTrackingAreaUpdateAccept(emmtrackingareaupdateaccept::EMMTrackingAreaUpdateAccept),
    EMMTrackingAreaUpdateComplete(emmtrackingareaupdatecomplete::EMMTrackingAreaUpdateComplete),
    EMMTrackingAreaUpdateReject(emmtrackingareaupdatereject::EMMTrackingAreaUpdateReject),
    EMMExtServiceRequest(emmextservicerequest::EMMExtServiceRequest),
    EMMCPServiceRequest(emmcpservicerequest::EMMCPServiceRequest),
    EMMServiceReject(emmservicereject::EMMServiceReject),
    EMMServiceAccept(emmserviceaccept::EMMServiceAccept),
    EMMGUTIReallocCommand(emmgutirealloccommand::EMMGUTIReallocCommand),
    EMMGUTIReallocComplete(emmgutirealloccomplete::EMMGUTIReallocComplete),
    EMMAuthenticationRequest(emmauthenticationrequest::EMMAuthenticationRequest),
    EMMAuthenticationResponse(emmauthenticationresponse::EMMAuthenticationResponse),
    EMMAuthenticationReject(emmauthenticationreject::EMMAuthenticationReject),
    EMMAuthenticationFailure(emmauthenticationfailure::EMMAuthenticationFailure),
    EMMIdentityRequest(emmidentityrequest::EMMIdentityRequest),
    EMMIdentityResponse(emmidentityresponse::EMMIdentityResponse),
    EMMSecurityModeCommand(emmsecuritymodecommand::EMMSecurityModeCommand),
    EMMSecurityModeComplete(emmsecuritymodecomplete::EMMSecurityModeComplete),
    EMMSecurityModeReject(emmsecuritymodereject::EMMSecurityModeReject),
    EMMStatus(emmstatus::EMMStatus),
    EMMInformation(emminformation::EMMInformation),
    EMMDLNASTransport(emmdlnastransport::EMMDLNASTransport),
    EMMULNASTransport(emmulnastransport::EMMULNASTransport),
    EMMCSServiceNotification(emmcsservicenotification::EMMCSServiceNotification),
    EMMDLGenericNASTransport(emmdlgenericnastransport::EMMDLGenericNASTransport),
    EMMULGenericNASTransport(emmulgenericnastransport::EMMULGenericNASTransport)
}

pub fn parse_emm_nas(bytes: &[u8]) -> Result<NASLTEMessage, DekuError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let mut reader = Reader::new(&mut cursor);
    let header = EMMHeader::from_reader_with_ctx(&mut reader, ())?;
    Ok(match header.emm_type {
        EMMType::AttachRequest => NASLTEMessage::EMMAttachRequest(emmattachrequest::EMMAttachRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachAccept => todo!(),
        EMMType::AttachComplete => todo!(),
        EMMType::AttachReject => todo!(),
        EMMType::DetachRequest => todo!(),
        EMMType::DetachAccept => todo!(),
        EMMType::TrackingAreaUpdateRequest => todo!(),
        EMMType::TrackingAreaUpdateAccept => todo!(),
        EMMType::TrackingAreaUpdateComplete => todo!(),
        EMMType::TrackingAreaUpdateReject => todo!(),
        EMMType::ExtendedServiceRequest => todo!(),
        EMMType::ControlPlaneServiceRequest => todo!(),
        EMMType::ServiceReject => todo!(),
        EMMType::ServiceAccept => todo!(),
        EMMType::GUTIReallocationCommand => todo!(),
        EMMType::GUTIReallocationComplete => todo!(),
        EMMType::AuthenticationRequest => todo!(),
        EMMType::AuthenticationResponse => todo!(),
        EMMType::AuthenticationReject => todo!(),
        EMMType::AuthenticationFailure => todo!(),
        EMMType::IdentityRequest => NASLTEMessage::EMMIdentityRequest(emmidentityrequest::EMMIdentityRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::IdentityResponse => todo!(),
        EMMType::SecurityModeCommand => todo!(),
        EMMType::SecurityModeComplete => todo!(),
        EMMType::SecurityModeReject => todo!(),
        EMMType::EMMStatus => todo!(),
        EMMType::EMMInformation => todo!(),
        EMMType::DownlinkNASTransport => todo!(),
        EMMType::UplinkNASTransport => todo!(),
        EMMType::CSServiceNotification => todo!(),
        EMMType::DownlinkGenericNASTransport => todo!(),
        EMMType::UplinkGenericNASTransport => todo!(),
    })
}

#[derive(DekuRead, DekuWrite, Debug)]
#[deku(id_type = "u8", bits = 4)]
enum ProtocolDiscriminator {
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

#[cfg(test)]
mod tests {
    use crate::nas::generated;

    use super::*;

    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn it_works() {
        // let bytes = decode_hex("07412208391185184409309005f0700000100030023ed031d127298080211001000010810600000000830600000000000d00000300ff0003130184000a000005000010005c0a009011034f18a6f15d0103c1000000000000");
        let bytes = decode_hex("075501");
        dbg!(parse_emm_nas(&bytes));
    }
}
