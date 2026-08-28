#!/usr/bin/env python3
"""
fix-encoding.py -- Repair UTF-8 BOM and CP1252 mojibake in text files.

Usage:
    python scripts/fix-encoding.py <file1> [file2 file3 ...]

Detects and repairs:
  - UTF-8 BOM (byte order mark) at start of file
  - CP1252/Latin-1 double-encoding mojibake:
      - em-dash (--) mis-encoded
      - smart apostrophe mis-encoded
      - left/right double quotes mis-encoded via cp1252/latin1
      - ellipsis/other C1 control chars from corrupted arrows
  - Runs scripts/check-utf8.ps1 (if available) after fixing.

Exit code: 0 if all files repaired successfully, 1 on failure.

Derived from the check-utf8.ps1 signature catalog in .clinerules/encoding.md.
"""

import sys
import os
import subprocess


# -- Mojibake signatures (mirrors check-utf8.ps1 construction) ----------

# em-dash (U+2014): CP1252 double-encode
EM_DASH_CP1252 = chr(0x00E2) + chr(0x20AC) + chr(0x201D)
# em-dash (latin1)
EM_DASH_LATIN1 = chr(0x00E2) + chr(0x20AC) + chr(0x0094)

# smart apostrophe (U+2019): CP1252
SMART_APOS_CP1252 = chr(0x00E2) + chr(0x20AC) + chr(0x2122)
# smart apostrophe (latin1)
SMART_APOS_LATIN1 = chr(0x00E2) + chr(0x20AC) + chr(0x0099)

# left double quote (U+201C): CP1252
LEFT_QUOTE_CP1252 = chr(0x00E2) + chr(0x20AC) + chr(0x0153)
# left double quote (latin1)
LEFT_QUOTE_LATIN1 = chr(0x00E2) + chr(0x20AC) + chr(0x009C)

# right double quote (U+201D): CP1252
RIGHT_QUOTE_CP1252 = chr(0x00E2) + chr(0x20AC) + chr(0x017E)
# right double quote (latin1)
RIGHT_QUOTE_LATIN1 = chr(0x00E2) + chr(0x20AC) + chr(0x009D)

# wide-A-tilde pattern: mis-encoded multiplication sign
WIDE_A_TILDE = chr(0x00C3) + chr(0x2014)

# replacement targets
EM_DASH = chr(0x2014)        # --
SMART_APOS = chr(0x2019)     # '
LEFT_QUOTE = chr(0x201C)     # left double quote
RIGHT_QUOTE = chr(0x201D)    # right double quote
MULTIPLY = chr(0x00D7)       # multiplication sign

# Priority-ordered replacements (longest-first)
REPLACEMENTS = [
    (EM_DASH_CP1252, EM_DASH),
    (EM_DASH_LATIN1, EM_DASH),
    (SMART_APOS_CP1252, SMART_APOS),
    (SMART_APOS_LATIN1, SMART_APOS),
    (LEFT_QUOTE_CP1252, LEFT_QUOTE),
    (LEFT_QUOTE_LATIN1, LEFT_QUOTE),
    (RIGHT_QUOTE_CP1252, RIGHT_QUOTE),
    (RIGHT_QUOTE_LATIN1, RIGHT_QUOTE),
    (WIDE_A_TILDE, MULTIPLY),
]


def fix_file(path: str) -> int:
    """Fix encoding for a single file. Returns number of repairs made."""
    with open(path, 'rb') as f:
        raw = f.read()

    original_len = len(raw)
    repairs = 0

    # 1. Strip UTF-8 BOM bytes if present
    if raw[:3] == b'\xef\xbb\xbf':
        raw = raw[3:]
        repairs += 1

    # 2. Decode as strict UTF-8
    try:
        text = raw.decode('utf-8')
    except UnicodeDecodeError as e:
        print(f"SKIP {path}: not valid UTF-8 ({e})")
        return 0

    # 3. Strip BOM character if present (U+FEFF at position 0)
    if text and text[0] == chr(0xFEFF):
        text = text[1:]
        repairs += 1

    # 4. Fix mojibake sequences
    for old, new in REPLACEMENTS:
        count = text.count(old)
        if count > 0:
            text = text.replace(old, new)
            repairs += count

    # 5. Handle incomplete mojibake: just U+00E2 U+20AC without third char
    # This is typically an ellipsis or similar corrupted 3-byte sequence.
    incomplete_pat = chr(0x00E2) + chr(0x20AC)
    while True:
        idx = text.find(incomplete_pat)
        if idx < 0:
            break
        if idx + 2 >= len(text):
            text = text[:idx] + '...'
            repairs += 1
            continue
        third = text[idx + 2]
        third_ord = ord(third)
        if third_ord == 0x2026 or third_ord == 0x00A6:
            # Ellipsis mojibake -- replace with ...
            text = text[:idx] + '...' + text[idx + 3:]
        else:
            # Unknown third char -- replace first two chars
            text = text[:idx] + '?' + text[idx + 1:]
        repairs += 1

    # 6. Write back
    new_bytes = text.encode('utf-8')
    with open(path, 'wb') as f:
        f.write(new_bytes)

    if repairs > 0:
        print(f"FIXED {path}: {repairs} repair(s), {original_len} -> {len(new_bytes)} bytes")
    else:
        print(f"CLEAN {path}: no repairs needed")

    return repairs


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(0)

    total_repairs = 0
    for path in args:
        if not os.path.isfile(path):
            print(f"SKIP {path}: not a file")
            continue
        total_repairs += fix_file(path)

    if total_repairs > 0:
        print(f"\nTotal: {total_repairs} repairs across {len(args)} file(s)")

        # Run encoding guard if available
        script_dir = os.path.dirname(os.path.abspath(__file__))
        check_script = os.path.join(script_dir, 'check-utf8.ps1')
        if os.path.isfile(check_script):
            print("\nRunning encoding guard...")
            repo_root = os.path.dirname(script_dir)
            result = subprocess.run(
                ['powershell', '-NoProfile', '-ExecutionPolicy', 'Bypass', check_script],
                capture_output=True, text=True, cwd=repo_root
            )
            print(result.stdout)
            if result.returncode != 0:
                print("WARNING: Encoding guard still reports issues after fix.")
                sys.exit(1)
            else:
                print("Encoding guard PASSED.")
    else:
        print("No repairs needed.")

    sys.exit(0)


if __name__ == '__main__':
    main()