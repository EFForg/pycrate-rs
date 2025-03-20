from collections import defaultdict
from scapy.utils import rdpcap
from pycrate_mobile.TS24007 import IE, Layer3E
from pycrate_core import elt
from pycrate_mobile import NASLTE
from typing import Any
import os
import sys
import json


GSMTAP_HDR_START = 28
GSMTAP_HDR_END = GSMTAP_HDR_START + 16
GSMTAP_TYPE_NAS = 18


def pycrate_obj_to_json(po: Any) -> Any:
    if isinstance(po, elt.Envelope):
        return pycrate_envelope_to_json(po)
    elif isinstance(po, elt.Atom):
        return pycrate_atom_to_json(po)
    else:
        raise ValueError('unknown pycrate obj', po)


def pycrate_atom_to_json(po: elt.Atom) -> Any:
    return {
        'name': po._name,
        'type': 'atom',
        'value': po.to_json(),
    }


def pycrate_envelope_to_json(po: elt.Envelope) -> Any:
    return po.to_json()
    return {
        'name': po._name,
        'type': 'envelope',
        'items': [pycrate_obj_to_json(item) for item in po._content]
    }


def parse_nas_packet(data: bytes) -> elt.Envelope:
    parsed = NASLTE.parse_NASLTE_MO(data)
    if parsed[0] is None:
        parsed = NASLTE.parse_NASLTE_MT(data)
    if parsed[0] is None:  # Not a NAS Packet
        raise TypeError("Not a nas packet")
    return parsed[0]


def read_nas_packets(input_file: os.DirEntry, output_dirname: str) -> None:
    packets = {}
    packet_types = defaultdict(lambda: 0)
    with open(input_file.path, 'rb') as f:
        pcap_file = rdpcap(f)
        for i, packet in enumerate(pcap_file):
            gsmtap_hdr = packet.load[GSMTAP_HDR_START:GSMTAP_HDR_END]
            gsmtap_type = gsmtap_hdr[2]
            packet_data = packet.load[GSMTAP_HDR_END:]
            if gsmtap_type == GSMTAP_TYPE_NAS:
                try:
                    packet = parse_nas_packet(packet_data)
                    packet_types[packet.__class__.__name__] += 1
                    packets[i] = json.loads(packet.to_json())
                except TypeError as e:
                    print(f"err on packet {i}: {e}")
    output_filename = os.path.join(
        output_dirname,
        input_file.name.replace('pcap', 'json')
    )
    with open(output_filename, 'w') as f:
        f.write(json.dumps(packets, indent=4))
    print(json.dumps(packet_types, indent=4))
    print(f"{input_file.path} -> {output_filename}")


def main() -> None:
    input_dirname = sys.argv[1]
    output_dirname = sys.argv[2]
    os.makedirs(output_dirname, exist_ok=True)
    for entry in os.scandir(input_dirname):
        read_nas_packets(entry, output_dirname)


main()
