#!/usr/bin/env python3
"""Exercise the real shell installer with local release fixtures and fake curl/uname."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.fake = self.root / 'commands'
        self.fake.mkdir()
        self.work = self.root / 'temporary downloads'
        self.work.mkdir()
        self.destination = self.root / 'install with spaces'
        self.destination.mkdir()
        self.binary = self.destination / 'paq'
        self.binary.write_text('existing installation\n')
        self.script('uname', 'case "$1" in -s) echo "$TEST_OS";; -m) echo "$TEST_ARCH";; esac')
        self.script('getconf', 'case "$1" in LONG_BIT) echo "${TEST_BITS:-64}";; GNU_LIBC_VERSION) echo "glibc 2.39";; *) exit 1;; esac')
        self.script('sysctl', 'echo "${TEST_ARM64:-0}"')
        self.script('curl', '''
output=''
previous=''
for arg do
  if [ "$previous" = --output ]; then output=$arg; fi
  previous=$arg
  url=$arg
done
printf '%s\n' "$url" >> "$TEST_FIXTURE/requests"
case "$url" in
  https://github.com/gregl83/paq/releases/latest)
    printf '%s' "$TEST_LATEST_URL" ;;
  "$TEST_URL/$TEST_ASSET"|"$TEST_URL/$TEST_ASSET.sha256")
    cp "$TEST_FIXTURE/${url##*/}" "$output" ;;
  *) echo "unexpected URL: $url" >&2; exit 1 ;;
esac
''')
        self.env = dict(os.environ, PATH=f'{self.fake}:{os.environ["PATH"]}',
                        PAQ_INSTALL_DIR=str(self.destination), PAQ_VERSION='latest',
                        TMPDIR=str(self.work),
                        TEST_OS='Linux', TEST_ARCH='x86_64',
                        TEST_URL='https://github.com/gregl83/paq/releases/download/v2.0.0',
                        TEST_LATEST_URL='https://github.com/gregl83/paq/releases/tag/v2.0.0',
                        TEST_FIXTURE=str(self.root))
        self.fixture()

    def script(self, name, content):
        path = self.fake / name
        path.write_text('#!/bin/sh\n' + content + '\n')
        path.chmod(0o755)

    def fixture(self, platform='ubuntu-x64', content=b'#!/bin/sh\necho paq-test\n', entry='paq'):
        self.asset = self.root / f'paq-{platform}.zip'
        with zipfile.ZipFile(self.asset, 'w') as archive:
            archive.writestr(entry, content)
        digest = hashlib.sha256(self.asset.read_bytes()).hexdigest()
        Path(str(self.asset) + '.sha256').write_text(f'{digest}  {self.asset.name}\n')
        self.env['TEST_ASSET'] = self.asset.name

    def run_installer(self, success=True, source=None, args=()):
        result = subprocess.run(['sh', '-s', '--', *args],
                                input=source if source is not None else (ROOT / 'install.sh').read_text(),
                                env=self.env, text=True, capture_output=True)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(self.binary.read_text(), 'existing installation\n')
        self.assertEqual(sorted(p.name for p in self.destination.iterdir()), ['paq'])
        self.assertEqual(list(self.work.iterdir()), [])
        return result

    def test_platforms_and_versions(self):
        for os_name, arch, platform in [('Linux', 'x86_64', 'ubuntu-x64'),
                                        ('Linux', 'i686', 'ubuntu-x86'),
                                        ('Darwin', 'x86_64', 'macos-x64'),
                                        ('Darwin', 'arm64', 'macos-arm64')]:
            for version in ['latest', '2.0.0', 'v2.0.0-rc.1']:
                with self.subTest(os=os_name, arch=arch, version=version):
                    self.fixture(platform)
                    self.env.update(TEST_OS=os_name, TEST_ARCH=arch, PAQ_VERSION=version)
                    self.env['TEST_URL'] = ('https://github.com/gregl83/paq/releases/download/v2.0.0'
                                            if version == 'latest' else
                                            'https://github.com/gregl83/paq/releases/download/v' + version.removeprefix('v'))
                    self.run_installer()
                    self.assertTrue(os.access(self.binary, os.X_OK))
                    self.assertIn('paq-test', self.binary.read_text())

    def test_rosetta_prefers_arm64(self):
        self.env.update(TEST_OS='Darwin', TEST_ARCH='x86_64', TEST_ARM64='1')
        self.fixture('macos-arm64')
        self.run_installer()

    def test_intel_mac_without_arm64_sysctl(self):
        self.env.update(TEST_OS='Darwin', TEST_ARCH='x86_64')
        self.script('sysctl', 'exit 1')
        self.fixture('macos-x64')
        self.run_installer()

    def test_32_bit_userspace_on_64_bit_kernel(self):
        self.env['TEST_BITS'] = '32'
        self.fixture('ubuntu-x86')
        self.run_installer()

    def test_unknown_userspace_bitness(self):
        self.env['TEST_BITS'] = 'unknown'
        self.run_installer(False)

    def test_latest_resolved_once(self):
        self.run_installer()
        requests = (self.root / 'requests').read_text().splitlines()
        self.assertEqual(requests, [
            'https://github.com/gregl83/paq/releases/latest',
            self.env['TEST_URL'] + '/' + self.asset.name,
            self.env['TEST_URL'] + '/' + self.asset.name + '.sha256',
        ])

    def test_invalid_latest_redirect(self):
        for url in ['https://example.com/releases/tag/v2.0.0',
                    'https://github.com/gregl83/paq/releases/latest',
                    'https://github.com/gregl83/paq/releases/tag/vgarbage']:
            self.env['TEST_LATEST_URL'] = url
            self.run_installer(False)

    def test_pinned_version_skips_latest(self):
        self.env['PAQ_VERSION'] = '2.0.0'
        self.run_installer()
        requests = (self.root / 'requests').read_text().splitlines()
        self.assertEqual(len(requests), 2)
        self.assertTrue(all('/download/v2.0.0/' in url for url in requests))

    def test_invalid_versions(self):
        for version in ['../../main', '1garbage', '1.2', '01.2.3', '1.2.3/extra']:
            self.env['PAQ_VERSION'] = version
            self.run_installer(False)

    def test_first_install(self):
        self.binary.unlink()
        self.destination.rmdir()
        self.run_installer()
        self.assertTrue(os.access(self.binary, os.X_OK))

    def test_unsupported_platform(self):
        for os_name, arch in [('Linux', 'aarch64'), ('FreeBSD', 'x86_64'), ('Windows', 'x86_64')]:
            self.env.update(TEST_OS=os_name, TEST_ARCH=arch)
            self.run_installer(False)

    def test_non_glibc(self):
        self.script('getconf', 'exit 1')
        self.run_installer(False)

    def test_tampered_archive(self):
        with self.asset.open('ab') as archive:
            archive.write(b'tampered')
        self.run_installer(False)

    def test_missing_checksum(self):
        Path(str(self.asset) + '.sha256').unlink()
        self.run_installer(False)

    def test_malformed_checksum(self):
        for checksum in ['bad', 'a' * 64 + '  other.zip\n', 'a' * 64 + f'  {self.asset.name}\n' * 2]:
            Path(str(self.asset) + '.sha256').write_text(checksum)
            self.run_installer(False)

    def test_archive_paths(self):
        for entry in ['../paq', '/paq', 'nested/paq']:
            self.fixture(entry=entry)
            self.run_installer(False)

    def test_unusable_binary(self):
        self.fixture(content=b'#!/bin/sh\nexit 127\n')
        self.run_installer(False)

    def test_download_failure(self):
        self.script('curl', 'exit 22')
        self.run_installer(False)

    def test_invalid_destination(self):
        for destination in ['/', 'relative/path']:
            self.env['PAQ_INSTALL_DIR'] = destination
            self.run_installer(False)

    def test_truncated_script(self):
        source = (ROOT / 'install.sh').read_text().rsplit('main "$@"', 1)[0]
        self.run_installer(source=source)
        self.assertEqual(self.binary.read_text(), 'existing installation\n')

    def test_help_and_unknown_option(self):
        self.run_installer(args=('--help',))
        self.run_installer(False, args=('--unknown',))


@unittest.skipUnless(sys.platform.startswith('linux'), 'Release publishing runs on Linux')
class ReleasePublicationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.assets = []
        for platform in ['ubuntu-x64', 'ubuntu-x86', 'macos-x64', 'macos-arm64',
                         'windows-x64', 'windows-x86']:
            name = f'paq-{platform}.zip'
            (self.root / name).write_bytes(b'release fixture')
            digest = hashlib.sha256(b'release fixture').hexdigest()
            (self.root / (name + '.sha256')).write_text(f'{digest}  {name}\n')
            self.assets.extend([name, name + '.sha256'])
        fake = self.root / 'commands'
        fake.mkdir()
        gh = fake / 'gh'
        gh.write_text('''#!/bin/sh
printf '%s\\n' "$*" >> "$TEST_LOG"
case "$1 $2" in
  'release view')
    case "$*" in
      *isDraft*)
        case "$TEST_STATE" in
          new) exit 1 ;;
          published) echo false ;;
          *) echo true ;;
        esac ;;
      *assets*)
        if [ "$TEST_STATE" != missing-remote ]; then
          printf '%s\\n' paq-*.zip paq-*.zip.sha256
        fi ;;
    esac ;;
  'release upload') [ "$TEST_STATE" != upload-failure ] ;;
  'release create'|'release edit') exit 0 ;;
  *) exit 1 ;;
esac
''')
        gh.chmod(0o755)
        self.log = self.root / 'requests'
        self.env = dict(os.environ, PATH=f'{fake}:{os.environ["PATH"]}',
                        TEST_LOG=str(self.log), TEST_STATE='new')

    def run_release(self, state='new', tag='v2.0.0', success=True):
        self.env['TEST_STATE'] = state
        self.log.unlink(missing_ok=True)
        result = subprocess.run(['bash', str(ROOT / 'bin/publish-release.sh'), tag],
                                cwd=self.root, env=self.env, text=True, capture_output=True)
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        calls = self.log.read_text().splitlines() if self.log.exists() else []
        edits = [call for call in calls if call.startswith('release edit')]
        self.assertEqual(len(edits), 1 if success else 0)
        if success:
            self.assertEqual(calls[-1], edits[0])
            upload = next(call for call in calls if call.startswith('release upload'))
            for asset in self.assets:
                self.assertIn(asset, upload.split())
        return calls

    def test_complete_release_published_last(self):
        calls = self.run_release()
        self.assertIn('--draft', next(call for call in calls if call.startswith('release create')))
        self.assertIn('--prerelease=false', calls[-1])

    def test_draft_can_be_retried(self):
        calls = self.run_release('draft')
        self.assertFalse(any(call.startswith('release create') for call in calls))

    def test_public_release_is_not_modified(self):
        calls = self.run_release('published', success=False)
        self.assertFalse(any(call.startswith('release upload') for call in calls))

    def test_incomplete_upload_never_published(self):
        for state in ['upload-failure', 'missing-remote']:
            with self.subTest(state=state):
                self.run_release(state, success=False)

    def test_missing_local_asset_never_uploaded(self):
        (self.root / self.assets[0]).unlink()
        self.assertEqual(self.run_release(success=False), [])

    def test_bad_local_checksum_never_uploaded(self):
        (self.root / self.assets[0]).write_bytes(b'tampered')
        self.assertEqual(self.run_release(success=False), [])

    def test_prerelease_stays_prerelease(self):
        calls = self.run_release(tag='v2.0.0-rc.1')
        self.assertIn('--prerelease=true', calls[-1])


if __name__ == '__main__':
    unittest.main()
