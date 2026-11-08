#!/usr/bin/env python3
"""Create an authenticated Sparkle feed from a packaged release, without exporting keys."""
import argparse
import datetime
import email.utils
import os
import plistlib
import subprocess
import tempfile
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
SPARKLE = "http://www.andymatuschak.org/xml-namespaces/sparkle"
# The installed bundle and signing account share this stable identity.
BUNDLE_ID = "io.devkit.desktop"
PUBLIC_KEY = "VMLqt73XeFU7d/CqPBorEl6j6zDLn+d66lPb2MsLiro="


def create(archive, url, notes, output, account):
    components = urlsplit(url)
    if (components.scheme != "https" or not components.hostname or components.username
            or components.password or components.query or components.fragment):
        raise ValueError("Use an immutable HTTPS archive URL without credentials, query or fragment")
    if "/latest/" in components.path:
        raise ValueError("The update archive must have a versioned URL")
    build_path = Path(os.environ.get("DEVLINE_SWIFT_BUILD_PATH", ROOT / "macos/.build"))
    tool_dir = (build_path / "artifacts/sparkle/Sparkle/bin").resolve(strict=True)
    signer = tool_dir / "sign_update"
    key = subprocess.check_output([tool_dir / "generate_keys", "--account", account, "-p"], text=True).strip()
    if key != PUBLIC_KEY:
        raise ValueError("The Keychain signing account does not match Devline's embedded public key")
    with zipfile.ZipFile(archive) as bundle:
        info = plistlib.loads(bundle.read("Devline.app/Contents/Info.plist"))
    if (info.get("CFBundleIdentifier") != BUNDLE_ID or info.get("SUPublicEDKey") != key
            or info.get("SURequireSignedFeed") is not True or info.get("SUVerifyUpdateBeforeExtraction") is not True):
        raise ValueError("Archive is not a Devline build configured to require signed updates")
    signature = subprocess.check_output([signer, "--account", account, "-p", archive], text=True).strip()
    subprocess.run([signer, "--account", account, "--verify", archive, signature], check=True)
    ET.register_namespace("sparkle", SPARKLE)
    rss = ET.Element("rss", version="2.0")
    channel = ET.SubElement(rss, "channel")
    ET.SubElement(channel, "title").text = "Devline Updates"
    ET.SubElement(channel, "language").text = "en"
    item = ET.SubElement(channel, "item")
    ET.SubElement(item, "title").text = "Devline " + info["CFBundleShortVersionString"]
    ET.SubElement(item, "pubDate").text = email.utils.format_datetime(datetime.datetime.now(datetime.timezone.utc))
    ET.SubElement(item, "description").text = notes.read_text()
    ET.SubElement(item, f"{{{SPARKLE}}}version").text = info["CFBundleVersion"]
    ET.SubElement(item, f"{{{SPARKLE}}}shortVersionString").text = info["CFBundleShortVersionString"]
    ET.SubElement(item, f"{{{SPARKLE}}}minimumSystemVersion").text = info["LSMinimumSystemVersion"]
    ET.SubElement(item, "enclosure", {
        "url": url, "length": str(archive.stat().st_size), "type": "application/octet-stream",
        f"{{{SPARKLE}}}edSignature": signature,
    })
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="devline-appcast-", dir=output.parent) as directory:
        temporary = Path(directory) / "appcast.xml"
        ET.indent(rss)
        ET.ElementTree(rss).write(temporary, encoding="utf-8", xml_declaration=True)
        subprocess.run([signer, "--account", account, temporary], check=True)
        subprocess.run([signer, "--account", account, "--verify", temporary], check=True)
        temporary.replace(output)
    print(f"Verified signed archive and feed: {output}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("url")
    parser.add_argument("--notes", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--account", default=BUNDLE_ID)
    args = parser.parse_args()
    create(args.archive.resolve(), args.url, args.notes, args.output.resolve(), args.account)
