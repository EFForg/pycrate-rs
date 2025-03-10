use deku::prelude::*;
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

use super::generated::emm::*;

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
pub enum EMMMessage {
    EMMAttachRequest(emmattachrequest::EMMAttachRequest),
    EMMAttachAccept(emmattachaccept::EMMAttachAccept),
    EMMAttachComplete(emmattachcomplete::EMMAttachComplete),
    EMMAttachReject(emmattachreject::EMMAttachReject),
    EMMDetachRequestMT(emmdetachrequestmt::EMMDetachRequestMT),
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

pub fn parse_emm_nas<R: Read+Seek>(emm_type: EMMType, mut reader: Reader<R>) -> Result<EMMMessage, DekuError> {
    Ok(match emm_type {
        EMMType::AttachRequest => EMMMessage::EMMAttachRequest(emmattachrequest::EMMAttachRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachAccept => EMMMessage::EMMAttachAccept(emmattachaccept::EMMAttachAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachComplete => EMMMessage::EMMAttachComplete(emmattachcomplete::EMMAttachComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachReject => EMMMessage::EMMAttachReject(emmattachreject::EMMAttachReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::DetachRequest => {
            let cursor = reader.into_inner();
            let bookmark = cursor.seek(SeekFrom::Current(0))
                .map_err(|err| DekuError::Io(err.kind()))?;
            let mut reader = Reader::new(cursor);
            if let Ok(mo_result) = emmdetachrequestmo::EMMDetachRequestMO::from_reader_with_ctx(&mut reader, ()) {
                EMMMessage::EMMDetachRequestMO(mo_result)
            } else {
                let cursor = reader.into_inner();
                cursor.seek(SeekFrom::Start(bookmark))
                    .map_err(|err| DekuError::Io(err.kind()))?;
                let mut reader = Reader::new(cursor);
                EMMMessage::EMMDetachRequestMT(emmdetachrequestmt::EMMDetachRequestMT::from_reader_with_ctx(&mut reader, ())?)
            }
        },
        EMMType::DetachAccept => EMMMessage::EMMDetachAccept(emmdetachaccept::EMMDetachAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateRequest => EMMMessage::EMMTrackingAreaUpdateRequest(emmtrackingareaupdaterequest::EMMTrackingAreaUpdateRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateAccept => EMMMessage::EMMTrackingAreaUpdateAccept(emmtrackingareaupdateaccept::EMMTrackingAreaUpdateAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateComplete => EMMMessage::EMMTrackingAreaUpdateComplete(emmtrackingareaupdatecomplete::EMMTrackingAreaUpdateComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateReject => EMMMessage::EMMTrackingAreaUpdateReject(emmtrackingareaupdatereject::EMMTrackingAreaUpdateReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ExtendedServiceRequest => EMMMessage::EMMExtServiceRequest(emmextservicerequest::EMMExtServiceRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ControlPlaneServiceRequest => EMMMessage::EMMCPServiceRequest(emmcpservicerequest::EMMCPServiceRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ServiceReject => EMMMessage::EMMServiceReject(emmservicereject::EMMServiceReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ServiceAccept => EMMMessage::EMMServiceAccept(emmserviceaccept::EMMServiceAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::GUTIReallocationCommand => EMMMessage::EMMGUTIReallocCommand(emmgutirealloccommand::EMMGUTIReallocCommand::from_reader_with_ctx(&mut reader, ())?),
        EMMType::GUTIReallocationComplete => EMMMessage::EMMGUTIReallocComplete(emmgutirealloccomplete::EMMGUTIReallocComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationRequest => EMMMessage::EMMAuthenticationRequest(emmauthenticationrequest::EMMAuthenticationRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationResponse => EMMMessage::EMMAuthenticationResponse(emmauthenticationresponse::EMMAuthenticationResponse::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationReject => EMMMessage::EMMAuthenticationReject(emmauthenticationreject::EMMAuthenticationReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationFailure => EMMMessage::EMMAuthenticationFailure(emmauthenticationfailure::EMMAuthenticationFailure::from_reader_with_ctx(&mut reader, ())?),
        EMMType::IdentityRequest => EMMMessage::EMMIdentityRequest(emmidentityrequest::EMMIdentityRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::IdentityResponse => EMMMessage::EMMIdentityResponse(emmidentityresponse::EMMIdentityResponse::from_reader_with_ctx(&mut reader, ())?),
        EMMType::SecurityModeCommand => EMMMessage::EMMSecurityModeCommand(emmsecuritymodecommand::EMMSecurityModeCommand::from_reader_with_ctx(&mut reader, ())?),
        EMMType::SecurityModeComplete => EMMMessage::EMMSecurityModeComplete(emmsecuritymodecomplete::EMMSecurityModeComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::SecurityModeReject => EMMMessage::EMMSecurityModeReject(emmsecuritymodereject::EMMSecurityModeReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::EMMStatus => EMMMessage::EMMStatus(emmstatus::EMMStatus::from_reader_with_ctx(&mut reader, ())?),
        EMMType::EMMInformation => EMMMessage::EMMInformation(emminformation::EMMInformation::from_reader_with_ctx(&mut reader, ())?),
        EMMType::DownlinkNASTransport => EMMMessage::EMMDLNASTransport(emmdlnastransport::EMMDLNASTransport::from_reader_with_ctx(&mut reader, ())?),
        EMMType::UplinkNASTransport => EMMMessage::EMMULNASTransport(emmulnastransport::EMMULNASTransport::from_reader_with_ctx(&mut reader, ())?),
        EMMType::CSServiceNotification => EMMMessage::EMMCSServiceNotification(emmcsservicenotification::EMMCSServiceNotification::from_reader_with_ctx(&mut reader, ())?),
        EMMType::DownlinkGenericNASTransport => EMMMessage::EMMDLGenericNASTransport(emmdlgenericnastransport::EMMDLGenericNASTransport::from_reader_with_ctx(&mut reader, ())?),
        EMMType::UplinkGenericNASTransport => EMMMessage::EMMULGenericNASTransport(emmulgenericnastransport::EMMULGenericNASTransport::from_reader_with_ctx(&mut reader, ())?),
    })
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
