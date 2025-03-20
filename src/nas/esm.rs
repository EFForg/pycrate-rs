use deku::prelude::*;
use std::io::{Read, Seek};
use serde::Serialize;

use super::generated::esm::{esm_act_dedi_eps_bearer_ctxt_accept::ESMActDediEPSBearerCtxtAccept, esm_act_dedi_eps_bearer_ctxt_reject::ESMActDediEPSBearerCtxtReject, esm_act_dedi_eps_bearer_ctxt_request::ESMActDediEPSBearerCtxtRequest, esm_act_default_eps_bearer_ctxt_accept::ESMActDefaultEPSBearerCtxtAccept, esm_act_default_eps_bearer_ctxt_reject::ESMActDefaultEPSBearerCtxtReject, esm_act_default_eps_bearer_ctxt_request::ESMActDefaultEPSBearerCtxtRequest, esm_bearer_resource_alloc_reject::ESMBearerResourceAllocReject, esm_bearer_resource_alloc_request::ESMBearerResourceAllocRequest, esm_bearer_resource_modif_reject::ESMBearerResourceModifReject, esm_bearer_resource_modif_request::ESMBearerResourceModifRequest, esm_data_transport::ESMDataTransport, esm_deact_eps_bearer_ctxt_accept::ESMDeactEPSBearerCtxtAccept, esm_deact_eps_bearer_ctxt_request::ESMDeactEPSBearerCtxtRequest, esm_dummy_message::ESMDummyMessage, esm_information_request::ESMInformationRequest, esm_information_response::ESMInformationResponse, esm_modify_eps_bearer_ctxt_accept::ESMModifyEPSBearerCtxtAccept, esm_modify_eps_bearer_ctxt_reject::ESMModifyEPSBearerCtxtReject, esm_modify_eps_bearer_ctxt_request::ESMModifyEPSBearerCtxtRequest, esm_notification::ESMNotification, esm_remote_ue_report::ESMRemoteUEReport, esm_remote_ue_response::ESMRemoteUEResponse, esm_status::ESMStatus, esmpdn_connectivity_reject::ESMPDNConnectivityReject, esmpdn_connectivity_request::ESMPDNConnectivityRequest, esmpdn_disconnect_reject::ESMPDNDisconnectReject, esmpdn_disconnect_request::ESMPDNDisconnectRequest};

#[derive(DekuRead, DekuWrite, Debug)]
#[deku(id_type = "u8")]
pub enum ESMType {
    #[deku(id = "193")] ActDefaultEPSBearerCtxtRequest,
    #[deku(id = "194")] ActDefaultEPSBearerCtxtAccept,
    #[deku(id = "195")] ActDefaultEPSBearerCtxtReject,
    #[deku(id = "197")] ActDediEPSBearerCtxtRequest,
    #[deku(id = "198")] ActDediEPSBearerCtxtAccept,
    #[deku(id = "199")] ActDediEPSBearerCtxtReject,
    #[deku(id = "201")] ModifyEPSBearerCtxtRequest,
    #[deku(id = "202")] ModifyEPSBearerCtxtAccept,
    #[deku(id = "203")] ModifyEPSBearerCtxtReject,
    #[deku(id = "205")] DeactEPSBearerCtxtRequest,
    #[deku(id = "206")] DeactEPSBearerCtxtAccept,
    #[deku(id = "208")] PDNConnectivityRequest,
    #[deku(id = "209")] PDNConnectivityReject,
    #[deku(id = "210")] PDNDisconnectRequest,
    #[deku(id = "211")] PDNDisconnectReject,
    #[deku(id = "212")] BearerResourceAllocRequest,
    #[deku(id = "213")] BearerResourceAllocReject,
    #[deku(id = "214")] BearerResourceModifRequest,
    #[deku(id = "215")] BearerResourceModifReject,
    #[deku(id = "217")] InformationRequest,
    #[deku(id = "218")] InformationResponse,
    #[deku(id = "219")] Notification,
    #[deku(id = "220")] DummyMessage,
    #[deku(id = "232")] Status,
    #[deku(id = "233")] RemoteUEReport,
    #[deku(id = "234")] RemoteUEResponse,
    #[deku(id = "235")] DataTransport
}

#[derive(Debug, Clone, Serialize)]
pub enum ESMMessage {
    ActDefaultEPSBearerCtxtRequest(ESMActDefaultEPSBearerCtxtRequest),
    ActDefaultEPSBearerCtxtAccept(ESMActDefaultEPSBearerCtxtAccept),
    ActDefaultEPSBearerCtxtReject(ESMActDefaultEPSBearerCtxtReject),
    ActDediEPSBearerCtxtRequest(ESMActDediEPSBearerCtxtRequest),
    ActDediEPSBearerCtxtAccept(ESMActDediEPSBearerCtxtAccept),
    ActDediEPSBearerCtxtReject(ESMActDediEPSBearerCtxtReject),
    ModifyEPSBearerCtxtRequest(ESMModifyEPSBearerCtxtRequest),
    ModifyEPSBearerCtxtAccept(ESMModifyEPSBearerCtxtAccept),
    ModifyEPSBearerCtxtReject(ESMModifyEPSBearerCtxtReject),
    DeactEPSBearerCtxtRequest(ESMDeactEPSBearerCtxtRequest),
    DeactEPSBearerCtxtAccept(ESMDeactEPSBearerCtxtAccept),
    PDNConnectivityRequest(ESMPDNConnectivityRequest),
    PDNConnectivityReject(ESMPDNConnectivityReject),
    PDNDisconnectRequest(ESMPDNDisconnectRequest),
    PDNDisconnectReject(ESMPDNDisconnectReject),
    BearerResourceAllocRequest(ESMBearerResourceAllocRequest),
    BearerResourceAllocReject(ESMBearerResourceAllocReject),
    BearerResourceModifRequest(ESMBearerResourceModifRequest),
    BearerResourceModifReject(ESMBearerResourceModifReject),
    InformationRequest(ESMInformationRequest),
    InformationResponse(ESMInformationResponse),
    Notification(ESMNotification),
    DummyMessage(ESMDummyMessage),
    Status(ESMStatus),
    RemoteUEReport(ESMRemoteUEReport),
    RemoteUEResponse(ESMRemoteUEResponse),
    DataTransport(ESMDataTransport),
}

pub fn parse_esm_nas<R: Read+Seek>(esm_type: ESMType, mut reader: Reader<R>) -> Result<ESMMessage, DekuError> {
    Ok(match esm_type {
        ESMType::ActDefaultEPSBearerCtxtRequest => ESMMessage::ActDefaultEPSBearerCtxtRequest(ESMActDefaultEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDefaultEPSBearerCtxtAccept => ESMMessage::ActDefaultEPSBearerCtxtAccept(ESMActDefaultEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDefaultEPSBearerCtxtReject => ESMMessage::ActDefaultEPSBearerCtxtReject(ESMActDefaultEPSBearerCtxtReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDediEPSBearerCtxtRequest => ESMMessage::ActDediEPSBearerCtxtRequest(ESMActDediEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDediEPSBearerCtxtAccept => ESMMessage::ActDediEPSBearerCtxtAccept(ESMActDediEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDediEPSBearerCtxtReject => ESMMessage::ActDediEPSBearerCtxtReject(ESMActDediEPSBearerCtxtReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ModifyEPSBearerCtxtRequest => ESMMessage::ModifyEPSBearerCtxtRequest(ESMModifyEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ModifyEPSBearerCtxtAccept => ESMMessage::ModifyEPSBearerCtxtAccept(ESMModifyEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ModifyEPSBearerCtxtReject => ESMMessage::ModifyEPSBearerCtxtReject(ESMModifyEPSBearerCtxtReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DeactEPSBearerCtxtRequest => ESMMessage::DeactEPSBearerCtxtRequest(ESMDeactEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DeactEPSBearerCtxtAccept => ESMMessage::DeactEPSBearerCtxtAccept(ESMDeactEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNConnectivityRequest => ESMMessage::PDNConnectivityRequest(ESMPDNConnectivityRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNConnectivityReject => ESMMessage::PDNConnectivityReject(ESMPDNConnectivityReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNDisconnectRequest => ESMMessage::PDNDisconnectRequest(ESMPDNDisconnectRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNDisconnectReject => ESMMessage::PDNDisconnectReject(ESMPDNDisconnectReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceAllocRequest => ESMMessage::BearerResourceAllocRequest(ESMBearerResourceAllocRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceAllocReject => ESMMessage::BearerResourceAllocReject(ESMBearerResourceAllocReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceModifRequest => ESMMessage::BearerResourceModifRequest(ESMBearerResourceModifRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceModifReject => ESMMessage::BearerResourceModifReject(ESMBearerResourceModifReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::InformationRequest => ESMMessage::InformationRequest(ESMInformationRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::InformationResponse => ESMMessage::InformationResponse(ESMInformationResponse::from_reader_with_ctx(&mut reader, ())?),
        ESMType::Notification => ESMMessage::Notification(ESMNotification::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DummyMessage => ESMMessage::DummyMessage(ESMDummyMessage::from_reader_with_ctx(&mut reader, ())?),
        ESMType::Status => ESMMessage::Status(ESMStatus::from_reader_with_ctx(&mut reader, ())?),
        ESMType::RemoteUEReport => ESMMessage::RemoteUEReport(ESMRemoteUEReport::from_reader_with_ctx(&mut reader, ())?),
        ESMType::RemoteUEResponse => ESMMessage::RemoteUEResponse(ESMRemoteUEResponse::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DataTransport => ESMMessage::DataTransport(ESMDataTransport::from_reader_with_ctx(&mut reader, ())?),
    })
}
