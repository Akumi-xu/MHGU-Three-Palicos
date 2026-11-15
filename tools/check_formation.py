"""Verify the placement detour preserves the native VFP height return via a tail branch."""
import re
import subprocess
import sys

dump = subprocess.check_output(
    [sys.argv[2] if len(sys.argv) > 2 else 'arm-none-eabi-objdump',
     '-d', '-C', sys.argv[1]], text=True
)
match = re.search(r'^\w+ <three_palicos::place_on_ground::__detour>:\n(.*?)(?=\n\w+ <)',
                  dump, re.M | re.S)
assert match, 'placement detour symbol missing'
instructions = [m[1].strip() for m in re.finditer(r'^\s*[0-9a-f]+:\s+[0-9a-f]+\s+(.+)$',
                                                match[1], re.M)
                if not m[1].lstrip().startswith('.')]
assert re.fullmatch(r'bx\s+(ip|r12)', instructions[-1]), (
    'Native VFP height must be returned through a tail branch; inspect this build', instructions[-5:]
)
print('PASS: ground placement tail branch preserves native VFP height return')
