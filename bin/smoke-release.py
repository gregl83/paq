#!/usr/bin/env python3
"""Test actual release ZIPs before publishing; only network downloads are substituted."""
import argparse
import hashlib
import os
from pathlib import Path
import platform
import struct
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def verify_architecture(data, target):
    if target.startswith('ubuntu-'):
        bits, machine = (2, 62) if target.endswith('x64') else (1, 3)
        require(data[:4] == b'\x7fELF' and data[4:6] == bytes([bits, 1])
                and struct.unpack_from('<H', data, 18)[0] == machine,
                f'Incorrect ELF architecture for {target}')
    elif target.startswith('macos-'):
        cpu = 0x0100000C if target.endswith('arm64') else 0x01000007
        require(struct.unpack_from('<II', data) == (0xFEEDFACF, cpu),
                f'Incorrect Mach-O architecture for {target}')
    else:
        offset = struct.unpack_from('<I', data, 60)[0]
        machine = 0x8664 if target.endswith('x64') else 0x014C
        require(data[:2] == b'MZ' and data[offset:offset + 4] == b'PE\0\0'
                and struct.unpack_from('<H', data, offset + 4)[0] == machine,
                f'Incorrect PE architecture for {target}')


def install_with_script(assets, archive, destination, work, version):
    commands = work / 'commands'
    commands.mkdir()
    curl = commands / 'curl'
    curl.write_text('''#!/bin/sh
set -eu
output=''
previous=''
for arg do
  if [ "$previous" = --output ]; then output=$arg; fi
  previous=$arg
  url=$arg
done
case "$url" in
  "$SMOKE_URL/$SMOKE_ARCHIVE"|"$SMOKE_URL/$SMOKE_ARCHIVE.sha256")
    cp "$SMOKE_ASSETS/${url##*/}" "$output" ;;
  *) echo "Unexpected release URL: $url" >&2; exit 1 ;;
esac
''')
    curl.chmod(0o755)
    env = dict(os.environ, PATH=f'{commands}:{os.environ["PATH"]}',
               PAQ_VERSION=version, PAQ_INSTALL_DIR=str(destination),
               SMOKE_URL=f'https://github.com/gregl83/paq/releases/download/v{version}',
               SMOKE_ASSETS=str(assets), SMOKE_ARCHIVE=archive.name)
    subprocess.run(['sh', str(ROOT / 'install.sh')], env=env, check=True, timeout=120)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('assets', type=Path)
    parser.add_argument('target', choices=['ubuntu-x64', 'ubuntu-x86', 'macos-x64',
                                         'macos-arm64', 'windows-x64', 'windows-x86'])
    parser.add_argument('version')
    args = parser.parse_args()
    system = {'ubuntu': 'Linux', 'macos': 'Darwin', 'windows': 'Windows'}[args.target.split('-')[0]]
    require(platform.system() == system, f'{args.target} must be tested on {system}')
    assets = args.assets.resolve()
    archive = assets / f'paq-{args.target}.zip'
    checksum = Path(str(archive) + '.sha256').read_text().split()
    require(checksum == [hashlib.sha256(archive.read_bytes()).hexdigest(), archive.name],
            'Release checksum mismatch')
    executable = 'paq.exe' if system == 'Windows' else 'paq'
    with zipfile.ZipFile(archive) as package:
        require(package.namelist() == [executable], 'Unexpected release archive contents')
        data = package.read(executable)
    verify_architecture(data, args.target)

    with tempfile.TemporaryDirectory(prefix='paq-release-') as directory:
        work = Path(directory)
        destination = work / 'install with spaces'
        binary = destination / executable
        # Windows uses manual downloads. On a 64-bit Linux runner, exercise the
        # x86 executable through its compatibility runtime; installer selection
        # for 32-bit userspace is covered separately in test-installer.py.
        manual = system == 'Windows' or (args.target == 'ubuntu-x86'
                                         and subprocess.check_output(['getconf', 'LONG_BIT'], text=True).strip() == '64')
        if manual:
            destination.mkdir()
            binary.write_bytes(data)
            binary.chmod(0o755)
        else:
            install_with_script(assets, archive, destination, work, args.version)
        require(binary.read_bytes() == data, 'Installed bytes differ from the release archive')
        version = subprocess.check_output([str(binary), '--version'], text=True, timeout=30).strip()
        require(version == f'paq {args.version}', f'Wrong release version: {version}')
        source = work / 'alpha'
        source.write_bytes(b'alpha-body')
        result = subprocess.check_output([str(binary), str(source)], text=True, timeout=30).strip()
        # Same cross-platform fixture as tests/bin/output.rs.
        require(result == '31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e',
                f'Incorrect file hash: {result}')
        print(f'Release smoke passed: {args.target}, {version}')


if __name__ == '__main__':
    main()
