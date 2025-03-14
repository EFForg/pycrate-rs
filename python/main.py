import os
from pycrate_mobile import NASLTE

from generator.modules import generate_module


def main(filepath: str):
    emm_classes = list(NASLTE.EMMTypeMOClasses.values())
    emm_classes.append(NASLTE.EMMTypeMTClasses[69])  # add in the MT version of DetachRequest
    generate_module(os.path.join(filepath, 'emm'), emm_classes, [
        '0748610bf602f8108003c8c2e65e9a5804e060c0405202f810c4c25c0a00570220003103e5e0341302f810040511035758a65d0100c1', # EMM TAU Request
    ])
    generate_module(os.path.join(filepath, 'esm'), NASLTE.ESMTypeClasses.values(), [
        '0202d9', # ESM Info Req
    ])


if __name__ == "__main__":
    import sys
    main(sys.argv[1])
