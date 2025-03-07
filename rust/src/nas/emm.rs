use deku::prelude::*;
use serde::Serialize;
use std::io::{Read, Seek};

use super::generated::*;

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

#[derive(Debug, Clone, Serialize)]
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

pub fn parse_emm_nas<R: Read+Seek>(emm_type: EMMType, reader: &mut Reader<R>) -> Result<NASLTEMessage, DekuError> {
    Ok(match emm_type {
        EMMType::AttachRequest => NASLTEMessage::EMMAttachRequest(emmattachrequest::EMMAttachRequest::from_reader_with_ctx(reader, ())?),
        EMMType::AttachAccept => NASLTEMessage::EMMAttachAccept(emmattachaccept::EMMAttachAccept::from_reader_with_ctx(reader, ())?),
        EMMType::AttachComplete => NASLTEMessage::EMMAttachComplete(emmattachcomplete::EMMAttachComplete::from_reader_with_ctx(reader, ())?),
        EMMType::AttachReject => NASLTEMessage::EMMAttachReject(emmattachreject::EMMAttachReject::from_reader_with_ctx(reader, ())?),
        EMMType::DetachRequest => NASLTEMessage::EMMDetachRequestMO(emmdetachrequestmo::EMMDetachRequestMO::from_reader_with_ctx(reader, ())?),
        EMMType::DetachAccept => NASLTEMessage::EMMDetachAccept(emmdetachaccept::EMMDetachAccept::from_reader_with_ctx(reader, ())?),
        EMMType::TrackingAreaUpdateRequest => NASLTEMessage::EMMTrackingAreaUpdateRequest(emmtrackingareaupdaterequest::EMMTrackingAreaUpdateRequest::from_reader_with_ctx(reader, ())?),
        EMMType::TrackingAreaUpdateAccept => NASLTEMessage::EMMTrackingAreaUpdateAccept(emmtrackingareaupdateaccept::EMMTrackingAreaUpdateAccept::from_reader_with_ctx(reader, ())?),
        EMMType::TrackingAreaUpdateComplete => NASLTEMessage::EMMTrackingAreaUpdateComplete(emmtrackingareaupdatecomplete::EMMTrackingAreaUpdateComplete::from_reader_with_ctx(reader, ())?),
        EMMType::TrackingAreaUpdateReject => NASLTEMessage::EMMTrackingAreaUpdateReject(emmtrackingareaupdatereject::EMMTrackingAreaUpdateReject::from_reader_with_ctx(reader, ())?),
        EMMType::ExtendedServiceRequest => NASLTEMessage::EMMExtServiceRequest(emmextservicerequest::EMMExtServiceRequest::from_reader_with_ctx(reader, ())?),
        EMMType::ControlPlaneServiceRequest => NASLTEMessage::EMMCPServiceRequest(emmcpservicerequest::EMMCPServiceRequest::from_reader_with_ctx(reader, ())?),
        EMMType::ServiceReject => NASLTEMessage::EMMServiceReject(emmservicereject::EMMServiceReject::from_reader_with_ctx(reader, ())?),
        EMMType::ServiceAccept => NASLTEMessage::EMMServiceAccept(emmserviceaccept::EMMServiceAccept::from_reader_with_ctx(reader, ())?),
        EMMType::GUTIReallocationCommand => NASLTEMessage::EMMGUTIReallocCommand(emmgutirealloccommand::EMMGUTIReallocCommand::from_reader_with_ctx(reader, ())?),
        EMMType::GUTIReallocationComplete => NASLTEMessage::EMMGUTIReallocComplete(emmgutirealloccomplete::EMMGUTIReallocComplete::from_reader_with_ctx(reader, ())?),
        EMMType::AuthenticationRequest => NASLTEMessage::EMMAuthenticationRequest(emmauthenticationrequest::EMMAuthenticationRequest::from_reader_with_ctx(reader, ())?),
        EMMType::AuthenticationResponse => NASLTEMessage::EMMAuthenticationResponse(emmauthenticationresponse::EMMAuthenticationResponse::from_reader_with_ctx(reader, ())?),
        EMMType::AuthenticationReject => NASLTEMessage::EMMAuthenticationReject(emmauthenticationreject::EMMAuthenticationReject::from_reader_with_ctx(reader, ())?),
        EMMType::AuthenticationFailure => NASLTEMessage::EMMAuthenticationFailure(emmauthenticationfailure::EMMAuthenticationFailure::from_reader_with_ctx(reader, ())?),
        EMMType::IdentityRequest => NASLTEMessage::EMMIdentityRequest(emmidentityrequest::EMMIdentityRequest::from_reader_with_ctx(reader, ())?),
        EMMType::IdentityResponse => NASLTEMessage::EMMIdentityResponse(emmidentityresponse::EMMIdentityResponse::from_reader_with_ctx(reader, ())?),
        EMMType::SecurityModeCommand => NASLTEMessage::EMMSecurityModeCommand(emmsecuritymodecommand::EMMSecurityModeCommand::from_reader_with_ctx(reader, ())?),
        EMMType::SecurityModeComplete => NASLTEMessage::EMMSecurityModeComplete(emmsecuritymodecomplete::EMMSecurityModeComplete::from_reader_with_ctx(reader, ())?),
        EMMType::SecurityModeReject => NASLTEMessage::EMMSecurityModeReject(emmsecuritymodereject::EMMSecurityModeReject::from_reader_with_ctx(reader, ())?),
        EMMType::EMMStatus => NASLTEMessage::EMMStatus(emmstatus::EMMStatus::from_reader_with_ctx(reader, ())?),
        EMMType::EMMInformation => NASLTEMessage::EMMInformation(emminformation::EMMInformation::from_reader_with_ctx(reader, ())?),
        EMMType::DownlinkNASTransport => NASLTEMessage::EMMDLNASTransport(emmdlnastransport::EMMDLNASTransport::from_reader_with_ctx(reader, ())?),
        EMMType::UplinkNASTransport => NASLTEMessage::EMMULNASTransport(emmulnastransport::EMMULNASTransport::from_reader_with_ctx(reader, ())?),
        EMMType::CSServiceNotification => NASLTEMessage::EMMCSServiceNotification(emmcsservicenotification::EMMCSServiceNotification::from_reader_with_ctx(reader, ())?),
        EMMType::DownlinkGenericNASTransport => NASLTEMessage::EMMDLGenericNASTransport(emmdlgenericnastransport::EMMDLGenericNASTransport::from_reader_with_ctx(reader, ())?),
        EMMType::UplinkGenericNASTransport => NASLTEMessage::EMMULGenericNASTransport(emmulgenericnastransport::EMMULGenericNASTransport::from_reader_with_ctx(reader, ())?),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
}
