"""Check actual Mac text-clipping imports and EPUB spine order."""
import plistlib
import struct
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / 'src/formats.py'


class ImportTests(unittest.TestCase):
    def convert(self, source, action, suffix='txt'):
        target = source.parent / ('converted.' + suffix)
        subprocess.run([sys.executable, str(SCRIPT), action, str(source), str(target)], check=True)
        return target

    def test_binary_plist_clipping_preserves_unicode(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'sample.textClipping'
            source.write_bytes(plistlib.dumps({'public.utf8-plain-text': 'Café 🎉'.encode()}, fmt=plistlib.FMT_BINARY))
            self.assertEqual(self.convert(source, 'textclipping').read_text(encoding='utf-8'), 'Café 🎉')

    def test_appledouble_resource_fork(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'legacy.textClipping'
            source.touch()
            text = 'Classic café'.encode('mac_roman')
            payload = struct.pack('>I', len(text)) + text
            type_list = struct.pack('>H4sHH', 0, b'TEXT', 0, 10)
            reference = struct.pack('>hH', 128, 0xffff) + b'\x00\x00\x00\x00' + b'\x00' * 4
            resource_map = b'\x00' * 24 + struct.pack('>HH', 28, 50) + type_list + reference
            resource = struct.pack('>IIII', 256, 256 + len(payload), len(payload), len(resource_map)) + b'\x00' * 240 + payload + resource_map
            sidecar = struct.pack('>II16sHIII', 0x00051607, 0x00020000, b'\x00' * 16, 1, 2, 38, len(resource)) + resource
            source.with_name('._' + source.name).write_bytes(sidecar)
            self.assertEqual(self.convert(source, 'textclipping').read_text(encoding='utf-8'), 'Classic café')

    def test_epub_round_trip_and_spine_order(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'sample.txt'
            source.write_text('First & café\nSecond <line>\n', encoding='utf-8')
            book = self.convert(source, 'make_epub', 'epub')
            with zipfile.ZipFile(book) as archive:
                self.assertEqual(archive.infolist()[0].filename, 'mimetype')
                self.assertEqual(archive.infolist()[0].compress_type, zipfile.ZIP_STORED)
                self.assertEqual(archive.read('mimetype'), b'application/epub+zip')
            text = self.convert(book, 'epub', 'html').read_text(encoding='utf-8')
            self.assertIn('First &amp; café', text)
            self.assertLess(text.index('First'), text.index('Second'))

    def test_gzip_decompresses_and_rejects_bombs(self):
        import gzip
        with tempfile.TemporaryDirectory() as tmp:
            small = Path(tmp) / 'notes.txt.gz'
            small.write_bytes(gzip.compress(b'hello'))
            self.assertEqual(self.convert(small, 'gzip').read_bytes(), b'hello')
            bomb = Path(tmp) / 'bomb.gz'
            with gzip.open(bomb, 'wb', compresslevel=9) as out:
                chunk = b'A' * (1 << 24)
                for _ in range(65):  # 1040 MiB of one byte, ~1 MiB compressed
                    out.write(chunk)
            target = Path(tmp) / 'bomb.out'
            result = subprocess.run([sys.executable, str(SCRIPT), 'gzip', str(bomb), str(target)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'unexpectedly large', result.stderr)


if __name__ == '__main__':
    unittest.main()
