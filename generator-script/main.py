import binascii
import os
from typing import Tuple, Dict, Optional
from scapy.utils import rdpcap
from pycrate_mobile import NASLTE
from pycrate_core import elt

from generator.modules import generate_module


def parse_nas_packet(data: bytes) -> elt.Envelope:
    parsed = NASLTE.parse_NASLTE_MO(data)
    if parsed[0] is None:
        parsed = NASLTE.parse_NASLTE_MT(data)
    if parsed[0] is None:  # Not a NAS Packet
        raise TypeError("Not a nas packet")
    return parsed[0]


GSMTAP_HDR_START = 28
GSMTAP_HDR_END = GSMTAP_HDR_START + 16
GSMTAP_TYPE_NAS = 18


def get_test_cases(pcap_dir_filepath: str) -> Tuple[list[str], list[str]]:
    types_to_skip = [
        'EMMServiceRequest',
        'EMMSecProtNASMessage',
    ]
    longest_testcase: Dict[str, str] = {}
    for entry in os.scandir(pcap_dir_filepath):
        with open(entry.path, 'rb') as f:
            pcap_file = rdpcap(f)
            for i, packet in enumerate(pcap_file):
                gsmtap_hdr = packet.load[GSMTAP_HDR_START:GSMTAP_HDR_END]
                gsmtap_type = gsmtap_hdr[2]
                packet_data = packet.load[GSMTAP_HDR_END:]
                packet_data_str = packet_data.hex()
                if gsmtap_type == GSMTAP_TYPE_NAS:
                    try:
                        packet = parse_nas_packet(packet_data)
                        type_name = packet.__class__.__name__
                        existing_testcase = longest_testcase.get(type_name, '')
                        if len(existing_testcase) < len(packet_data_str):
                            longest_testcase[type_name] = packet_data_str
                    except TypeError as e:
                        print(f"err on packet {i}: {e}")
    emm_tests = []
    esm_tests = []
    for type_name, testcase in longest_testcase.items():
        if type_name in types_to_skip:
            continue
        if type_name.startswith("EMM"):
            emm_tests.append(testcase)
        elif type_name.startswith("ESM"):
            esm_tests.append(testcase)
        else:
            print(f'unexpected packet type {type_name}')
    return (emm_tests, esm_tests)


def main(output_filepath: str, pcap_dir_filepath: Optional[str]):
    if pcap_dir_filepath is None:
        emm_tests, esm_tests = [], []
    else:
        emm_tests, esm_tests = get_test_cases(pcap_dir_filepath)
    emm_classes = list(NASLTE.EMMTypeMOClasses.values())
    emm_classes.append(NASLTE.EMMTypeMTClasses[69])  # add in the MT version of DetachRequest
    generate_module(os.path.join(output_filepath, 'emm'), emm_classes, [
        '075501', # EMM IMSI identity request
        '0748610bf602f8108003c8c2e65e9a5804e060c0405202f810c4c25c0a00570220003103e5e0341302f810040511035758a65d0100c1', # EMM TAU Request
        '075e23093395684292874145f0', # EMM SMCompl
        '074300035200c2', # EMM Attach Complete
        '074c6005f4c2e65e9a57022000', # EMM Ext Serv Request
        '074a', # EMM TAU Complete
        '07632009011d00010007913386094000f01101830a816000000000000005d4f29cae00', # EMM NAS transport + SMS CP-DATA
        '0745630bf602f8108003c8c2e65e9a', # EMM Detach Request MO
        '074d707800040200e86f6703091011570233c9d1' # EMM CP Service Request
    ] + emm_tests)
    generate_module(os.path.join(output_filepath, 'esm'), NASLTE.ESMTypeClasses.values(), [
        '0202d9', # ESM Info Req
        '0202da2807066f72616e6765', # ESM Info Resp
    ] + esm_tests)


if __name__ == "__main__":
    import sys
    output_filepath = sys.argv[1]
    if len(sys.argv) == 2:
        pcap_dir_filepath = None
    else:
        pcap_dir_filepath = sys.argv[2]
    main(output_filepath, pcap_dir_filepath)
