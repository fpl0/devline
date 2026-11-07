"""Small manifest insertions that preserve the project's original source bytes."""
import xml.etree.ElementTree as ET
from xml.parsers import expat
from xml.sax.saxutils import escape


def add_maven_dependencies(path, packages):
    source = path.read_bytes()
    root = ET.fromstring(source)
    namespace = root.tag.partition('}')[0] + '}' if root.tag.startswith('{') else ''
    dependencies = root.find(namespace + 'dependencies')
    known = set() if dependencies is None else {
        (item.findtext(namespace + 'groupId'), item.findtext(namespace + 'artifactId'))
        for item in dependencies.findall(namespace + 'dependency')
    }
    additions = [(name.split(':'), package['version']) for name, package in packages.items()
                 if tuple(name.split(':')) not in known]
    if not additions:
        return
    parser = expat.ParserCreate()
    stack, positions = [], {}

    def start(name, _attributes):
        if not stack:
            positions['root_name'] = name
        elif len(stack) == 1 and name.split(':')[-1] == 'dependencies':
            positions['name'] = name
            positions['start'] = parser.CurrentByteIndex
        stack.append(name)

    def end(name):
        if len(stack) == 2 and name.split(':')[-1] == 'dependencies':
            positions['end'] = parser.CurrentByteIndex
        if len(stack) == 1:
            positions['root_end'] = parser.CurrentByteIndex
        stack.pop()

    parser.StartElementHandler, parser.EndElementHandler = start, end
    parser.Parse(source, True)
    prefix = positions['root_name'].rpartition(':')[0]
    prefix = prefix + ':' if prefix else ''
    newline = '\r\n' if b'\r\n' in source else '\n'
    lines = []
    for (group, artifact), version in additions:
        lines.append(f'    <{prefix}dependency>')
        for key, value in [('groupId', group), ('artifactId', artifact), ('version', version)]:
            lines.append(f'      <{prefix}{key}>{escape(value)}</{prefix}{key}>')
        lines.append(f'    </{prefix}dependency>')
    content = (newline + newline.join(lines) + newline + '  ').encode()
    if 'start' in positions:
        start = positions['start']
        quote = None
        opening_end = start
        for index in range(start, len(source)):
            character = source[index]
            if quote:
                if character == quote:
                    quote = None
            elif character in (ord('"'), ord("'")):
                quote = character
            elif character == ord('>'):
                opening_end = index
                break
        if source[start:opening_end].rstrip().endswith(b'/'):
            slash = source.rfind(b'/', start, opening_end)
            source = source[:slash] + b'>' + content + ('</' + positions['name'] + '>').encode() + source[opening_end + 1:]
        else:
            end = positions['end']
            source = source[:end] + content + source[end:]
    else:
        insertion = (newline + '  <' + prefix + 'dependencies>').encode() + content + ('</' + prefix + 'dependencies>' + newline).encode()
        end = positions['root_end']
        source = source[:end] + insertion + source[end:]
    # Validate the edited document before publishing even this staged copy.
    ET.fromstring(source)
    path.write_bytes(source)
