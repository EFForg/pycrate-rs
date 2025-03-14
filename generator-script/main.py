import os
from pycrate_mobile import NASLTE

from generator.modules import generate_module


def main(filepath: str):
    emm_classes = list(NASLTE.EMMTypeMOClasses.values())
    emm_classes.append(NASLTE.EMMTypeMTClasses[69])  # add in the MT version of DetachRequest
    generate_module(os.path.join(filepath, 'emm'), emm_classes, [
        '0748610bf602f8108003c8c2e65e9a5804e060c0405202f810c4c25c0a00570220003103e5e0341302f810040511035758a65d0100c1', # EMM TAU Request
        '075e23093395684292874145f0', # EMM SMCompl
        '074300035200c2', # EMM Attach Complete
        # 'c7060500', # EMM Serv Request
        '074c6005f4c2e65e9a57022000', # EMM Ext Serv Request
        '074a', # EMM TAU Complete
        '07632009011d00010007913386094000f01101830a816000000000000005d4f29cae00', # EMM NAS transport + SMS CP-DATA
        '0745630bf602f8108003c8c2e65e9a', # EMM Detach Request MO
        '074d707800040200e86f6703091011570233c9d1' # EMM CP Service Request
    ])
    generate_module(os.path.join(filepath, 'esm'), NASLTE.ESMTypeClasses.values(), [
        '0202d9', # ESM Info Req
        '0202da2807066f72616e6765', # ESM Info Resp
    ])


if __name__ == "__main__":
    import sys
    main(sys.argv[1])
