#!/usr/bin/env python3

import os
import sys
import subprocess

CONFIG = """\
arm_64bit=1
kernel_address={:#x}
"""

assert(len(sys.argv) == 2)

def get_entry_point(elf):
    header = subprocess.check_output(["readelf", "-h", elf],
                                     universal_newlines=True)
    for l in header.splitlines():
        if "Entry point address:" in l:
            return int(l.strip().split(":")[1], 16)

    raise Exception("Failed to find entry point")

txt = CONFIG.format(get_entry_point(sys.argv[1]))
config = os.path.join(os.path.dirname(sys.argv[1]), "config.txt")

with open(config, "w") as fd:
    fd.write(txt)
