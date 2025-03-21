use deku::prelude::*;
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

use super::generated::emm::{
    emm_attach_accept::EMMAttachAccept, emm_attach_complete::EMMAttachComplete,
    emm_attach_reject::EMMAttachReject, emm_attach_request::EMMAttachRequest,
    emm_authentication_failure::EMMAuthenticationFailure,
    emm_authentication_reject::EMMAuthenticationReject,
    emm_authentication_request::EMMAuthenticationRequest,
    emm_authentication_response::EMMAuthenticationResponse, emm_detach_accept::EMMDetachAccept,
    emm_detach_request_mo::EMMDetachRequestMO, emm_detach_request_mt::EMMDetachRequestMT,
    emm_ext_service_request::EMMExtServiceRequest, emm_identity_request::EMMIdentityRequest,
    emm_identity_response::EMMIdentityResponse, emm_information::EMMInformation,
    emm_security_mode_command::EMMSecurityModeCommand,
    emm_security_mode_complete::EMMSecurityModeComplete,
    emm_security_mode_reject::EMMSecurityModeReject, emm_service_accept::EMMServiceAccept,
    emm_service_reject::EMMServiceReject, emm_status::EMMStatus,
    emm_tracking_area_update_accept::EMMTrackingAreaUpdateAccept,
    emm_tracking_area_update_complete::EMMTrackingAreaUpdateComplete,
    emm_tracking_area_update_reject::EMMTrackingAreaUpdateReject,
    emm_tracking_area_update_request::EMMTrackingAreaUpdateRequest,
    emmcp_service_request::EMMCPServiceRequest,
    emmcs_service_notification::EMMCSServiceNotification,
    emmdl_generic_nas_transport::EMMDLGenericNASTransport, emmdlnas_transport::EMMDLNASTransport,
    emmguti_realloc_command::EMMGUTIReallocCommand,
    emmguti_realloc_complete::EMMGUTIReallocComplete,
    emmul_generic_nas_transport::EMMULGenericNASTransport, emmulnas_transport::EMMULNASTransport,
};

#[derive(DekuRead, DekuWrite, Debug)]
#[deku(id_type = "u8")]
pub enum EMMType {
    #[deku(id = 65)] AttachRequest,
    #[deku(id = 66)] AttachAccept,
    #[deku(id = 67)] AttachComplete,
    #[deku(id = 68)] AttachReject,
    #[deku(id = 69)] DetachRequest,
    #[deku(id = 70)] DetachAccept,
    #[deku(id = 72)] TrackingAreaUpdateRequest,
    #[deku(id = 73)] TrackingAreaUpdateAccept,
    #[deku(id = 74)] TrackingAreaUpdateComplete,
    #[deku(id = 75)] TrackingAreaUpdateReject,
    #[deku(id = 76)] ExtendedServiceRequest,
    #[deku(id = 77)] ControlPlaneServiceRequest,
    #[deku(id = 78)] ServiceReject,
    #[deku(id = 79)] ServiceAccept,
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
    EMMAttachRequest(EMMAttachRequest),
    EMMAttachAccept(EMMAttachAccept),
    EMMAttachComplete(EMMAttachComplete),
    EMMAttachReject(EMMAttachReject),
    EMMDetachRequestMT(EMMDetachRequestMT),
    EMMDetachRequestMO(EMMDetachRequestMO),
    EMMDetachAccept(EMMDetachAccept),
    EMMTrackingAreaUpdateRequest(EMMTrackingAreaUpdateRequest),
    EMMTrackingAreaUpdateAccept(EMMTrackingAreaUpdateAccept),
    EMMTrackingAreaUpdateComplete(EMMTrackingAreaUpdateComplete),
    EMMTrackingAreaUpdateReject(EMMTrackingAreaUpdateReject),
    EMMExtServiceRequest(EMMExtServiceRequest),
    EMMCPServiceRequest(EMMCPServiceRequest),
    EMMServiceReject(EMMServiceReject),
    EMMServiceAccept(EMMServiceAccept),
    EMMGUTIReallocCommand(EMMGUTIReallocCommand),
    EMMGUTIReallocComplete(EMMGUTIReallocComplete),
    EMMAuthenticationRequest(EMMAuthenticationRequest),
    EMMAuthenticationResponse(EMMAuthenticationResponse),
    EMMAuthenticationReject(EMMAuthenticationReject),
    EMMAuthenticationFailure(EMMAuthenticationFailure),
    EMMIdentityRequest(EMMIdentityRequest),
    EMMIdentityResponse(EMMIdentityResponse),
    EMMSecurityModeCommand(EMMSecurityModeCommand),
    EMMSecurityModeComplete(EMMSecurityModeComplete),
    EMMSecurityModeReject(EMMSecurityModeReject),
    EMMStatus(EMMStatus),
    EMMInformation(EMMInformation),
    EMMDLNASTransport(EMMDLNASTransport),
    EMMULNASTransport(EMMULNASTransport),
    EMMCSServiceNotification(EMMCSServiceNotification),
    EMMDLGenericNASTransport(EMMDLGenericNASTransport),
    EMMULGenericNASTransport(EMMULGenericNASTransport),
}

pub fn parse_emm_nas<R: Read + Seek>(
    emm_type: EMMType,
    mut reader: Reader<R>,
) -> Result<EMMMessage, DekuError> {
    Ok(match emm_type {
        EMMType::AttachRequest => EMMMessage::EMMAttachRequest(EMMAttachRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachAccept => EMMMessage::EMMAttachAccept(EMMAttachAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachComplete => EMMMessage::EMMAttachComplete(EMMAttachComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AttachReject => EMMMessage::EMMAttachReject(EMMAttachReject::from_reader_with_ctx(&mut reader, ())?),
        // DetachRequests are overloaded depending on whether the message is MO
        // (mobile originated) or MT (mobile terminated). Simply try both, and
        // return whichever one successfully parses
        EMMType::DetachRequest => {
            // save a bookmark to the beginning of the message in case we need
            // to retry
            let cursor = reader.into_inner();
            let bookmark = cursor
                .seek(SeekFrom::Current(0))
                .map_err(|err| DekuError::Io(err.kind()))?;
            let mut reader = Reader::new(cursor);

            // attempt an MO parse
            if let Ok(mo_result) = EMMDetachRequestMO::from_reader_with_ctx(&mut reader, ()) {
                EMMMessage::EMMDetachRequestMO(mo_result)
            } else {
                // rewind, then attempt an MT parse
                let cursor = reader.into_inner();
                cursor
                    .seek(SeekFrom::Start(bookmark))
                    .map_err(|err| DekuError::Io(err.kind()))?;
                let mut reader = Reader::new(cursor);
                EMMMessage::EMMDetachRequestMT(EMMDetachRequestMT::from_reader_with_ctx(
                    &mut reader,
                    (),
                )?)
            }
        },
        EMMType::DetachAccept => EMMMessage::EMMDetachAccept(EMMDetachAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateRequest => EMMMessage::EMMTrackingAreaUpdateRequest(EMMTrackingAreaUpdateRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateAccept => EMMMessage::EMMTrackingAreaUpdateAccept(EMMTrackingAreaUpdateAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateComplete => EMMMessage::EMMTrackingAreaUpdateComplete(EMMTrackingAreaUpdateComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::TrackingAreaUpdateReject => EMMMessage::EMMTrackingAreaUpdateReject(EMMTrackingAreaUpdateReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ExtendedServiceRequest => EMMMessage::EMMExtServiceRequest(EMMExtServiceRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ControlPlaneServiceRequest => EMMMessage::EMMCPServiceRequest(EMMCPServiceRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ServiceReject => EMMMessage::EMMServiceReject(EMMServiceReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::ServiceAccept => EMMMessage::EMMServiceAccept(EMMServiceAccept::from_reader_with_ctx(&mut reader, ())?),
        EMMType::GUTIReallocationCommand => EMMMessage::EMMGUTIReallocCommand(EMMGUTIReallocCommand::from_reader_with_ctx(&mut reader, ())?),
        EMMType::GUTIReallocationComplete => EMMMessage::EMMGUTIReallocComplete(EMMGUTIReallocComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationRequest => EMMMessage::EMMAuthenticationRequest(EMMAuthenticationRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationResponse => EMMMessage::EMMAuthenticationResponse(EMMAuthenticationResponse::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationReject => EMMMessage::EMMAuthenticationReject(EMMAuthenticationReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::AuthenticationFailure => EMMMessage::EMMAuthenticationFailure(EMMAuthenticationFailure::from_reader_with_ctx(&mut reader, ())?),
        EMMType::IdentityRequest => EMMMessage::EMMIdentityRequest(EMMIdentityRequest::from_reader_with_ctx(&mut reader, ())?),
        EMMType::IdentityResponse => EMMMessage::EMMIdentityResponse(EMMIdentityResponse::from_reader_with_ctx(&mut reader, ())?),
        EMMType::SecurityModeCommand => EMMMessage::EMMSecurityModeCommand(EMMSecurityModeCommand::from_reader_with_ctx(&mut reader, ())?),
        EMMType::SecurityModeComplete => EMMMessage::EMMSecurityModeComplete(EMMSecurityModeComplete::from_reader_with_ctx(&mut reader, ())?),
        EMMType::SecurityModeReject => EMMMessage::EMMSecurityModeReject(EMMSecurityModeReject::from_reader_with_ctx(&mut reader, ())?),
        EMMType::EMMStatus => EMMMessage::EMMStatus(EMMStatus::from_reader_with_ctx(&mut reader, ())?),
        EMMType::EMMInformation => EMMMessage::EMMInformation(EMMInformation::from_reader_with_ctx(&mut reader, ())?),
        EMMType::DownlinkNASTransport => EMMMessage::EMMDLNASTransport(EMMDLNASTransport::from_reader_with_ctx(&mut reader, ())?),
        EMMType::UplinkNASTransport => EMMMessage::EMMULNASTransport(EMMULNASTransport::from_reader_with_ctx(&mut reader, ())?),
        EMMType::CSServiceNotification => EMMMessage::EMMCSServiceNotification(EMMCSServiceNotification::from_reader_with_ctx(&mut reader, ())?),
        EMMType::DownlinkGenericNASTransport => EMMMessage::EMMDLGenericNASTransport(EMMDLGenericNASTransport::from_reader_with_ctx(&mut reader, ())?),
        EMMType::UplinkGenericNASTransport => EMMMessage::EMMULGenericNASTransport(EMMULGenericNASTransport::from_reader_with_ctx(&mut reader, ())?),
    })
}
