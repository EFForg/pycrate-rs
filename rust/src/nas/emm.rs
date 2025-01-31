use super::layer3::Type1V;

use deku::prelude::*;

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

#[derive(Debug, DekuRead)]
struct EMMIdentityRequest {
    pub header: EMMHeader,
    #[deku(pad_bits_before = "4")]
    pub id_type: Type1V<IDType>,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
enum IDType {
    #[deku(id = 0)] NoIdentity,
    #[deku(id = 1)] IMSI,
    #[deku(id = 2)] IMEI,
    #[deku(id = 3)] IMEISV,
    #[deku(id = 4)] TMSI,
    #[deku(id = 5)] TMGI,
    #[deku(id = 6)] FFU,
}

#[derive(DekuRead, DekuWrite)]
struct UERadioCapIDDelInd {
    #[deku(pad_bits_before="1")]
    pub del_request: DelRequest,
}

#[derive(DekuRead, DekuWrite)]
#[deku(id_type = "u8", bits = 3)]
enum DelRequest {
    #[deku(id = 0)] NotRequested,
    #[deku(id = 1)] Requested,
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

    #[test]
    fn it_works() {
        let bytes = decode_hex("07412208391185184409309005f0700000100030023ed031d127298080211001000010810600000000830600000000000d00000300ff0003130184000a000005000010005c0a009011034f18a6f15d0103c1000000000000");
        let req = EMMIdentityRequest::from_bytes((&bytes, 0)).unwrap();
        dbg!(req);
    }
}
