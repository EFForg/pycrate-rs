use deku::prelude::*;
use std::io::{Read, Seek};
use serde::Serialize;

use super::generated::esm::*;

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
    ActDefaultEPSBearerCtxtRequest(esmactdefaultepsbearerctxtrequest::ESMActDefaultEPSBearerCtxtRequest),
    ActDefaultEPSBearerCtxtAccept(esmactdefaultepsbearerctxtaccept::ESMActDefaultEPSBearerCtxtAccept),
    ActDefaultEPSBearerCtxtReject(esmactdefaultepsbearerctxtreject::ESMActDefaultEPSBearerCtxtReject),
    ActDediEPSBearerCtxtRequest(esmactdediepsbearerctxtrequest::ESMActDediEPSBearerCtxtRequest),
    ActDediEPSBearerCtxtAccept(esmactdediepsbearerctxtaccept::ESMActDediEPSBearerCtxtAccept),
    ActDediEPSBearerCtxtReject(esmactdediepsbearerctxtreject::ESMActDediEPSBearerCtxtReject),
    ModifyEPSBearerCtxtRequest(esmmodifyepsbearerctxtrequest::ESMModifyEPSBearerCtxtRequest),
    ModifyEPSBearerCtxtAccept(esmmodifyepsbearerctxtaccept::ESMModifyEPSBearerCtxtAccept),
    ModifyEPSBearerCtxtReject(esmmodifyepsbearerctxtreject::ESMModifyEPSBearerCtxtReject),
    DeactEPSBearerCtxtRequest(esmdeactepsbearerctxtrequest::ESMDeactEPSBearerCtxtRequest),
    DeactEPSBearerCtxtAccept(esmdeactepsbearerctxtaccept::ESMDeactEPSBearerCtxtAccept),
    PDNConnectivityRequest(esmpdnconnectivityrequest::ESMPDNConnectivityRequest),
    PDNConnectivityReject(esmpdnconnectivityreject::ESMPDNConnectivityReject),
    PDNDisconnectRequest(esmpdndisconnectrequest::ESMPDNDisconnectRequest),
    PDNDisconnectReject(esmpdndisconnectreject::ESMPDNDisconnectReject),
    BearerResourceAllocRequest(esmbearerresourceallocrequest::ESMBearerResourceAllocRequest),
    BearerResourceAllocReject(esmbearerresourceallocreject::ESMBearerResourceAllocReject),
    BearerResourceModifRequest(esmbearerresourcemodifrequest::ESMBearerResourceModifRequest),
    BearerResourceModifReject(esmbearerresourcemodifreject::ESMBearerResourceModifReject),
    InformationRequest(esminformationrequest::ESMInformationRequest),
    InformationResponse(esminformationresponse::ESMInformationResponse),
    Notification(esmnotification::ESMNotification),
    DummyMessage(esmdummymessage::ESMDummyMessage),
    Status(esmstatus::ESMStatus),
    RemoteUEReport(esmremoteuereport::ESMRemoteUEReport),
    RemoteUEResponse(esmremoteueresponse::ESMRemoteUEResponse),
    DataTransport(esmdatatransport::ESMDataTransport),
}

pub fn parse_esm_nas<R: Read+Seek>(esm_type: ESMType, mut reader: Reader<R>) -> Result<ESMMessage, DekuError> {
    Ok(match esm_type {
        ESMType::ActDefaultEPSBearerCtxtRequest => ESMMessage::ActDefaultEPSBearerCtxtRequest(esmactdefaultepsbearerctxtrequest::ESMActDefaultEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDefaultEPSBearerCtxtAccept => ESMMessage::ActDefaultEPSBearerCtxtAccept(esmactdefaultepsbearerctxtaccept::ESMActDefaultEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDefaultEPSBearerCtxtReject => ESMMessage::ActDefaultEPSBearerCtxtReject(esmactdefaultepsbearerctxtreject::ESMActDefaultEPSBearerCtxtReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDediEPSBearerCtxtRequest => ESMMessage::ActDediEPSBearerCtxtRequest(esmactdediepsbearerctxtrequest::ESMActDediEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDediEPSBearerCtxtAccept => ESMMessage::ActDediEPSBearerCtxtAccept(esmactdediepsbearerctxtaccept::ESMActDediEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ActDediEPSBearerCtxtReject => ESMMessage::ActDediEPSBearerCtxtReject(esmactdediepsbearerctxtreject::ESMActDediEPSBearerCtxtReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ModifyEPSBearerCtxtRequest => ESMMessage::ModifyEPSBearerCtxtRequest(esmmodifyepsbearerctxtrequest::ESMModifyEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ModifyEPSBearerCtxtAccept => ESMMessage::ModifyEPSBearerCtxtAccept(esmmodifyepsbearerctxtaccept::ESMModifyEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::ModifyEPSBearerCtxtReject => ESMMessage::ModifyEPSBearerCtxtReject(esmmodifyepsbearerctxtreject::ESMModifyEPSBearerCtxtReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DeactEPSBearerCtxtRequest => ESMMessage::DeactEPSBearerCtxtRequest(esmdeactepsbearerctxtrequest::ESMDeactEPSBearerCtxtRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DeactEPSBearerCtxtAccept => ESMMessage::DeactEPSBearerCtxtAccept(esmdeactepsbearerctxtaccept::ESMDeactEPSBearerCtxtAccept::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNConnectivityRequest => ESMMessage::PDNConnectivityRequest(esmpdnconnectivityrequest::ESMPDNConnectivityRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNConnectivityReject => ESMMessage::PDNConnectivityReject(esmpdnconnectivityreject::ESMPDNConnectivityReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNDisconnectRequest => ESMMessage::PDNDisconnectRequest(esmpdndisconnectrequest::ESMPDNDisconnectRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::PDNDisconnectReject => ESMMessage::PDNDisconnectReject(esmpdndisconnectreject::ESMPDNDisconnectReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceAllocRequest => ESMMessage::BearerResourceAllocRequest(esmbearerresourceallocrequest::ESMBearerResourceAllocRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceAllocReject => ESMMessage::BearerResourceAllocReject(esmbearerresourceallocreject::ESMBearerResourceAllocReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceModifRequest => ESMMessage::BearerResourceModifRequest(esmbearerresourcemodifrequest::ESMBearerResourceModifRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::BearerResourceModifReject => ESMMessage::BearerResourceModifReject(esmbearerresourcemodifreject::ESMBearerResourceModifReject::from_reader_with_ctx(&mut reader, ())?),
        ESMType::InformationRequest => ESMMessage::InformationRequest(esminformationrequest::ESMInformationRequest::from_reader_with_ctx(&mut reader, ())?),
        ESMType::InformationResponse => ESMMessage::InformationResponse(esminformationresponse::ESMInformationResponse::from_reader_with_ctx(&mut reader, ())?),
        ESMType::Notification => ESMMessage::Notification(esmnotification::ESMNotification::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DummyMessage => ESMMessage::DummyMessage(esmdummymessage::ESMDummyMessage::from_reader_with_ctx(&mut reader, ())?),
        ESMType::Status => ESMMessage::Status(esmstatus::ESMStatus::from_reader_with_ctx(&mut reader, ())?),
        ESMType::RemoteUEReport => ESMMessage::RemoteUEReport(esmremoteuereport::ESMRemoteUEReport::from_reader_with_ctx(&mut reader, ())?),
        ESMType::RemoteUEResponse => ESMMessage::RemoteUEResponse(esmremoteueresponse::ESMRemoteUEResponse::from_reader_with_ctx(&mut reader, ())?),
        ESMType::DataTransport => ESMMessage::DataTransport(esmdatatransport::ESMDataTransport::from_reader_with_ctx(&mut reader, ())?),
    })
}
