"""Export the saved Azuki file icon gallery and its standalone SVG candidates.

Run: python3 mascots_playground/scripts/generate_file_icon_gallery.py
Requires only Python 3's standard library. All artwork and the Seti reference
font are preserved in sources/file_icon_gallery/; nothing is downloaded.
"""

from copy import deepcopy
from pathlib import Path
import re
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "sources/file_icon_gallery"
SVG_NAMESPACE = "http://www.w3.org/2000/svg"
ET.register_namespace("", SVG_NAMESPACE)


def read_palettes(fragment):
    palettes = {}
    for name, selector in [
        ("dark", '#azuki-file-icon-gallery .product'),
        ("light", '#azuki-file-icon-gallery[data-appearance="light"] .product'),
    ]:
        declaration = re.search(re.escape(selector) + r"\s*\{([^}]+)\}", fragment)
        if declaration is None:
            raise ValueError(f"Missing {name} palette in the gallery source")
        palettes[name] = dict(re.findall(r"--([a-z-]+):\s*(#[0-9a-fA-F]+)",
                                         declaration[1]))
    return palettes


def read_symbols(fragment):
    match = re.search(r'<svg class="symbols".*?</svg>', fragment, re.DOTALL)
    if match is None:
        raise ValueError("Missing SVG symbols in the gallery source")
    tree = ET.fromstring(match[0])
    return {symbol.attrib["id"]: symbol
            for symbol in tree.iter(f"{{{SVG_NAMESPACE}}}symbol")}


def expand_symbol(symbol_id, symbols, parents=()):
    if symbol_id in parents:
        raise ValueError(f"Circular SVG reference: {symbol_id}")
    elements = []
    for child in symbols[symbol_id]:
        if child.tag == f"{{{SVG_NAMESPACE}}}use":
            elements.extend(expand_symbol(child.attrib["href"].removeprefix("#"),
                                          symbols, parents + (symbol_id,)))
        else:
            elements.append(deepcopy(child))
    return elements


def export_candidates(fragment):
    palettes = read_palettes(fragment)
    symbols = read_symbols(fragment)
    destination = ROOT / "svg/file_icons"
    destination.mkdir(parents=True, exist_ok=True)
    for symbol_id, symbol in symbols.items():
        name = symbol_id.removeprefix("azuki-")
        for appearance, palette in palettes.items():
            svg = ET.Element(f"{{{SVG_NAMESPACE}}}svg", {
                "width": "16", "height": "16", "viewBox": symbol.attrib["viewBox"],
            })
            svg.extend(expand_symbol(symbol_id, symbols))
            for element in svg.iter():
                for attribute, value in element.attrib.items():
                    element.set(attribute, re.sub(r"var\(--([a-z-]+)\)",
                                                 lambda match: palette[match[1]], value))
            ET.indent(svg, space="  ")
            output = ET.tostring(svg, encoding="unicode") + "\n"
            (destination / f"{name}-{appearance}.svg").write_text(output,
                                                               encoding="utf-8")
    return len(symbols) * len(palettes)


def export_gallery(fragment):
    shell = (SOURCE / "shell.html").read_text(encoding="utf-8")
    notice = (SOURCE / "SETI_NOTICE.txt").read_text(encoding="utf-8")
    document = shell.replace("<!-- FILE_ICON_GALLERY -->", fragment.rstrip())
    document = document.replace("<!-- SETI_NOTICE -->", "<!--\n" + notice + "-->")
    destination = ROOT / "previews/file_icon_gallery.html"
    destination.write_text(document, encoding="utf-8")
    return destination


def main():
    fragment = (SOURCE / "fragment.html").read_text(encoding="utf-8")
    count = export_candidates(fragment)
    gallery = export_gallery(fragment)
    print(f"Generated {gallery} and {count} SVG candidates in svg/file_icons/.")


if __name__ == "__main__":
    main()
