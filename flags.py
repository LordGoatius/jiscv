#!/usr/bin/env python3
with open('.shell.bin.o', 'r+b') as f:
    f.seek(0x30)
    f.write(b'\x05\x00\x00\x00')

print("Flags set to 0x5")
