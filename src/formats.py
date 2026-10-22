"""Embedded standard-library import adapters; never extracts archive members to disk."""
import html
import plistlib
import posixpath
import struct
import sys
import zipfile
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
from pathlib import Path


class Text(HTMLParser):
    def __init__(self):
        super().__init__()
        self.parts = []
        self.hidden = 0

    def handle_starttag(self, tag, attrs):
        if tag in ('script', 'style'):
            self.hidden += 1
        if tag in ('p', 'div', 'br', 'h1', 'h2', 'h3', 'li', 'tr'):
            self.parts.append('\n')

    def handle_endtag(self, tag):
        if tag in ('script', 'style'):
            self.hidden = max(0, self.hidden - 1)
        if tag in ('p', 'div', 'h1', 'h2', 'h3', 'li', 'tr'):
            self.parts.append('\n')

    def handle_data(self, text):
        if not self.hidden:
            self.parts.append(text)


def epub(source):
    from urllib.parse import unquote
    with zipfile.ZipFile(source) as book:
        container = ET.fromstring(book.read('META-INF/container.xml'))
        root = next(e for e in container.iter() if e.tag.endswith('rootfile')).attrib['full-path']
        package = ET.fromstring(book.read(root))
        manifest = {e.attrib['id']: e.attrib['href'] for e in package.iter() if e.tag.endswith('}item')}
        chapters = []
        total = 0
        for ref in package.iter():
            if not ref.tag.endswith('}itemref'):
                continue
            name = posixpath.normpath(posixpath.join(posixpath.dirname(root), unquote(manifest[ref.attrib['idref']].split('#')[0])))
            info = book.getinfo(name)
            total += info.file_size
            if total > 64 * 1024 * 1024:
                raise ValueError('EPUB text exceeds 64 MiB')
            parser = Text()
            parser.feed(book.read(name).decode('utf-8-sig'))
            chapters.append(html.escape(''.join(parser.parts).strip()))
        if not any(chapters):
            raise ValueError('EPUB contains no readable chapters')
        return '<!doctype html><html><head><meta charset="utf-8"></head><body>' + ''.join('<p style="white-space: pre-wrap">' + c.replace('\n', '<br>') + '</p>' for c in chapters) + '</body></html>'


def clipping(source):
    data = source.read_bytes()
    try:
        values = plistlib.loads(data)
        if isinstance(values, dict):
            for key in ('public.utf8-plain-text', 'public.utf16-plain-text', 'NSStringPboardType', 'text', 'Text'):
                value = values.get(key)
                if isinstance(value, str):
                    return value
                if isinstance(value, bytes):
                    return value.decode('utf-16' if 'utf16' in key else 'utf-8')
    except (plistlib.InvalidFileException, ValueError):
        pass
    # Finder text clippings commonly carry their text in a resource fork.
    resource = source.with_name('._' + source.name)
    if resource.is_file():
        apple = resource.read_bytes()
        if len(apple) >= 26 and apple[:4] == b'\x00\x05\x16\x07':
            for i in range(struct.unpack_from('>H', apple, 24)[0]):
                ident, offset, length = struct.unpack_from('>III', apple, 26 + i * 12)
                if ident == 2:
                    data = apple[offset:offset + length]
                    break
    # Classic resource fork header + resource map type/reference tables.
    if len(data) >= 16:
        try:
            doff, moff, dlen, mlen = struct.unpack_from('>IIII', data)
            if moff + mlen <= len(data) and doff + dlen <= len(data):
                types = moff + struct.unpack_from('>H', data, moff + 24)[0]
                for i in range(struct.unpack_from('>H', data, types)[0] + 1):
                    at = types + 2 + i * 8
                    tag, count, refs = struct.unpack_from('>4sHH', data, at)
                    if tag not in (b'TEXT', b'utxt', b'utf8'):
                        continue
                    ref = types + refs
                    offset = int.from_bytes(data[ref + 5:ref + 8], 'big') + doff
                    length = struct.unpack_from('>I', data, offset)[0]
                    return data[offset + 4:offset + 4 + length].decode({b'TEXT': 'mac_roman', b'utxt': 'utf-16-be', b'utf8': 'utf-8'}[tag])
        except (struct.error, UnicodeError, IndexError):
            pass
    if data and b'\x00' not in data:
        return data.decode('utf-8-sig')
    raise ValueError('Text clipping has no readable text. Include its AppleDouble (._) sidecar when copying from macOS.')


action, src, dst = sys.argv[1:]
source = Path(src)
if action == 'make_epub':
    text = source.read_text(encoding='utf-8-sig')
    paragraphs = ''.join('<p>' + html.escape(line) + '</p>' for line in text.splitlines() if line.strip())
    with zipfile.ZipFile(dst, 'w') as book:
        book.writestr('mimetype', 'application/epub+zip', compress_type=zipfile.ZIP_STORED)
        book.writestr('META-INF/container.xml', '<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>')
        book.writestr('content.opf', '''<?xml version="1.0" encoding="UTF-8"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">urn:uuid:''' + __import__('uuid').uuid4().hex + '''</dc:identifier><dc:title>Converted document</dc:title><dc:language>en</dc:language><meta property="dcterms:modified">''' + __import__('datetime').datetime.now(__import__('datetime').timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ') + '''</meta></metadata><manifest><item id="text" href="text.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/></manifest><spine><itemref idref="text"/></spine></package>''')
        book.writestr('text.xhtml', '<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>Converted document</title></head><body>' + paragraphs + '</body></html>')
        book.writestr('nav.xhtml', '<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><ol><li><a href="text.xhtml">Document</a></li></ol></nav></body></html>')
elif action == 'gzip':
    import gzip
    import shutil
    import tarfile
    if not tarfile.is_tarfile(source):
        with gzip.open(source, 'rb') as compressed, open(dst, 'wb') as output:
            shutil.copyfileobj(compressed, output)
else:
    Path(dst).write_text(epub(source) if action == 'epub' else clipping(source), encoding='utf-8')
