"""Write pip's pinned requirements for the wheels in one directory."""

from email.parser import BytesParser
from hashlib import file_digest
from pathlib import Path
from sys import argv
from zipfile import ZipFile


def requirement(wheel):
    with ZipFile(wheel) as archive:
        name = next(name for name in archive.namelist() if name.endswith(".dist-info/METADATA"))
        metadata = BytesParser().parsebytes(archive.read(name), headersonly=True)
    with wheel.open("rb") as stream:
        digest = file_digest(stream, "sha256").hexdigest()
    return f"{metadata['Name']}=={metadata['Version']} --hash=sha256:{digest}"


if __name__ == "__main__":
    for wheel in sorted(Path(argv[1]).glob("*.whl")):
        print(requirement(wheel))
