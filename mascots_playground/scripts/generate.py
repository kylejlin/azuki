"""Regenerate the Azuki imagery experiments and their comparison boards.

Run from any directory: python3 mascots_playground/scripts/generate.py
Dependencies: Python 3, NumPy, Pillow, and rsvg-convert on PATH.
"""

from pathlib import Path
import hashlib
import json
import shutil
import subprocess

from PIL import Image, ImageDraw, ImageFont

from render_models import ROOT, read_glb, render


INK = "#292d36"
CREAM = "#f6efde"
RED = "#a8463e"
GOLD = "#c29b62"
GREEN = "#637961"


def svg_document(width, height, content, title, description):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title description">
<title id="title">{title}</title><desc id="description">{description}</desc>
{content}
</svg>'''


def antlers(compact=False, color=GOLD, outline=INK):
    if compact:
        paths = ["M208 269 182 201 157 168 151 111 M181 201 208 167 216 131 M157 168 118 157 101 127",
                 "M300 269 328 204 351 164 359 107 M329 204 301 171 295 133 M351 164 393 152 411 121"]
        return f'<g fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="{" ".join(paths)}" stroke="{outline}" stroke-width="27"/><path d="{" ".join(paths)}" stroke="{color}" stroke-width="17"/></g>'
    paths = ["M211 274 180 208 153 141 124 55 118 25 M153 141 188 100 200 57 M180 208 116 184 75 140 62 100 M116 184 118 142 M75 140 40 136 M141 106 100 80 83 52 M188 100 219 90",
             "M302 274 340 210 368 144 395 58 399 24 M368 144 335 103 321 61 M340 210 403 186 438 141 447 101 M403 186 404 145 M438 141 473 136 M381 105 425 78 442 49 M335 103 307 93"]
    return f'<g fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="{" ".join(paths)}" stroke="{outline}" stroke-width="23"/><path d="{" ".join(paths)}" stroke="{color}" stroke-width="14"/></g>'


def bean_body(book=False):
    body = f'''<path d="m197 389-34 26 7 31 36 17 35-33-6-29Z" fill="{INK}"/>
<path d="m291 396-18 35 30 31 40-13 3-31-28-26Z" fill="{INK}"/>
<path d="M249 252C207 250 172 280 165 326c-8 47 20 86 83 117 57-16 92-57 92-102 0-48-38-88-77-88Z" fill="{RED}" stroke="{INK}" stroke-width="8" stroke-linejoin="round"/>
<path d="m249 256-45 53-18 64 25 35 37 31-11-77 28-109Z" fill="#bf5d50"/>
<path d="m265 254 36 57 15 62-27 45 26-17 21-37-1-39-31-51Z" fill="#8b3634"/>
<path d="m186 319 14-10 14 4 9 16-2 22-13 10-15-7-10-17Z" fill="{INK}"/>
<path d="m284 318 15-5 14 10 5 20-7 17-16 3-13-13-3-18Z" fill="{INK}"/>
<path d="m211 274 17-9" fill="none" stroke="#e58b70" stroke-width="7" stroke-linecap="round"/>'''
    if book:
        body += f'''<path d="M169 374q28-13 79 10 52-23 91-9l-13 63q-38-13-78 4-43-21-77-7Z" fill="{CREAM}" stroke="{INK}" stroke-width="7" stroke-linejoin="round"/>
<path d="m248 384 0 57m-62-46q25-2 44 9m-43 6 42 12m36-18q24-9 49-7m-49 25 40-9" fill="none" stroke="#baa88c" stroke-width="4" stroke-linecap="round"/>
<path d="m169 390-12 8 8 17 16-4m153-20 12 9-8 16-15-4" fill="{RED}" stroke="{INK}" stroke-width="6" stroke-linejoin="round"/>'''
    return body


def branchkeeper(book=False, compact=False):
    return antlers(compact) + bean_body(book)


def sword():
    return f'''<g transform="translate(119 371) rotate(27)">
<path d="M-8-131 7-154 22-131 18-12-4-12Z" fill="#c0c9c9" stroke="{INK}" stroke-width="6" stroke-linejoin="round"/>
<path d="m7-143 0 125" stroke="#f5f5e9" stroke-width="4"/>
<path d="M-19-13h52v13h-52Zm19 13h13v35h-13Z" fill="{GOLD}" stroke="{INK}" stroke-width="6" stroke-linejoin="round"/>
</g>'''


def compact_mark(dark=False):
    antler_color = GOLD if dark else INK
    return f'''<g fill="none" stroke="{antler_color}" stroke-width="32" stroke-linecap="round" stroke-linejoin="round">
<path d="m190 208-40-75-9-66m27 99-63-35m215 77 42-75 9-66m-28 99 64-35"/>
</g>
<path d="M253 144c-91-5-141 62-141 145 0 71 51 126 141 157 90-31 147-93 147-163 0-79-59-134-133-139Z" fill="{RED}"/>
<path d="M253 144c-65 23-99 96-89 165 6 51 34 98 89 137-90-31-141-86-141-157 0-83 50-150 141-145Z" fill="#bb5a4e"/>
<ellipse cx="194" cy="282" rx="22" ry="29" fill="{INK}"/>
<ellipse cx="315" cy="282" rx="22" ry="29" fill="{INK}"/>'''


def tiny_mark():
    # The antlers deliberately give way to the bean at favicon sizes.
    return f'''<path d="M8 1C4 .6 1.4 3.8 1.4 8c0 3.4 2.3 5.7 6.6 7 4.3-1.3 6.6-4 6.6-7.4C14.6 3.6 11.8 1 8 1Z" fill="{RED}"/>
<path d="M8 1C5.1 3.8 4.3 10 8 15c-4.3-1.3-6.6-3.6-6.6-7C1.4 3.8 4 .6 8 1Z" fill="#c46756"/>
<path d="M4 6h2v3H4Zm6 0h2v3h-2Z" fill="#171d25"/>'''


def field_notes():
    return f'''<circle cx="400" cy="387" r="300" fill="#e9e7cf"/>
<path d="M128 663q256-46 539 0" fill="none" stroke="#d5cbb3" stroke-width="5"/>
<path d="m197 652 36-64 182-22 158 23 39 64Z" fill="#b0b49b"/>
<path d="m233 588 89 23 93-45m-93 45-15 50m108-95 61 65 97-42" fill="none" stroke="#919b82" stroke-width="3"/>
<g transform="translate(68 52) scale(1.29)">{branchkeeper(book=True, compact=True)}</g>
<g fill="none" stroke="{GREEN}" stroke-width="6" stroke-linecap="round"><path d="m152 650-10-48m9 28-28-16m27 2 19-26m447 60 16-67m-12 45 29-23m-23 0-13-17"/></g>
<g transform="translate(617 187) rotate(-14)"><path d="M0 0q-28-42-55-29-11 17 55 29-17 35-44 32-19-13 44-32 27-40 52-27 10 19-52 27 22 33 44 31 18-13-44-31Z" fill="#a2b6b0"/>
<path d="M0-8 0 39" stroke="{INK}" stroke-width="6" stroke-linecap="round"/><circle cy="-10" r="7" fill="{INK}"/></g>'''


def bridge_scene(dark=False):
    bg, hill, distant, water = ("#202a30", "#526e60", "#374c49", "#334c53") if dark else (CREAM, "#96aa86", "#d3d7b8", "#b7c9c3")
    moon = "#e6d5a7" if dark else "#e7c486"
    content = f'''<rect width="1600" height="560" fill="{bg}"/>
<circle cx="1240" cy="145" r="64" fill="{moon}"/>
<path d="M0 248q190-110 380 17 176-140 405-18 201-129 439-17 177-99 376-39v369H0Z" fill="{distant}"/>
<path d="M0 386q256-189 521-52 240-91 490 15 333-180 589-69v280H0Z" fill="{hill}"/>
<path d="M496 560q215-198 351-190 126 19 336 190Z" fill="{water}"/>
<path d="M373 410q414-173 839-7l-9 79q-90-27-159-27-177-153-346-6-161 8-307 47Z" fill="#ad9675"/>
<path d="M391 410q404-154 806-3" fill="none" stroke="#e0c9a1" stroke-width="22"/>
<path d="M391 435q404-154 806-3" fill="none" stroke="#685e51" stroke-width="6"/>
<g stroke="#685e51" stroke-width="4" opacity=".65"><path d="m465 399 6 54m89-82 7 58m85-80 5 57m89-69 1 58m97-57-3 58m95-45-9 55m99-37-11 53m89-31-9 52"/></g>
<path d="m1040 174 0 165" stroke="{INK}" stroke-width="7" stroke-linecap="round"/>
<path d="M1044 178q35-17 78 2l-11 28 15 24q-43-18-82-1Z" fill="{RED}" stroke="{INK}" stroke-width="4"/>
<path d="M1076 191c-16 0-17 22-1 28 17-6 16-27 1-28Z" fill="{CREAM}"/>
<g transform="translate(631 78) scale(.62)">{branchkeeper()}</g>
<g transform="translate(420 164) scale(.46)">{sword()}{bean_body()}</g>
<g transform="translate(883 184) scale(.43)">{bean_body(book=True)}</g>
<g fill="none" stroke="{INK}" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"><path d="m180 352 0-127m0 65-41-36m41 5 35-36m-35 10-26-24m1204 134 0-161m0 93-45-39m45 9 46-45m-46 16-23-27"/></g>
<g fill="{GOLD}"><ellipse cx="143" cy="251" rx="12" ry="22" transform="rotate(-43 143 251)"/><ellipse cx="211" cy="225" rx="12" ry="22" transform="rotate(37 211 225)"/><ellipse cx="153" cy="207" rx="10" ry="20" transform="rotate(-36 153 207)"/><ellipse cx="1318" cy="243" rx="13" ry="24" transform="rotate(-43 1318 243)"/><ellipse cx="1400" cy="216" rx="13" ry="24" transform="rotate(40 1400 216)"/></g>
<g fill="{CREAM}" opacity=".75"><path d="m115 434 2-10 2 10 10 2-10 2-2 10-2-10-10-2Z"/><path d="m1452 364 2-10 2 10 10 2-10 2-2 10-2-10-10-2Z"/></g>'''
    if dark:
        content += f'<g fill="{CREAM}" opacity=".65"><circle cx="198" cy="98" r="2"/><circle cx="369" cy="153" r="2"/><circle cx="963" cy="97" r="2"/><circle cx="1437" cy="88" r="2"/><circle cx="1012" cy="161" r="2"/></g>'
    return content


def export_svg(name, width, height, content, title, description, raster_width=None):
    target = ROOT / "svg" / f"{name}.svg"
    target.write_text(svg_document(width, height, content, title, description))
    subprocess.run(["rsvg-convert", "--width", str(raster_width or width * 2),
                    "--output", str(ROOT / "png" / f"{name}.png"), str(target)], check=True)


def font(size, serif=False):
    path = "/System/Library/Fonts/Supplemental/Georgia.ttf" if serif else "/System/Library/Fonts/SFNS.ttf"
    try:
        return ImageFont.truetype(path, size)
    except OSError:
        return ImageFont.truetype("DejaVuSerif.ttf" if serif else "DejaVuSans.ttf", size)


def composite_contained(board, image, box):
    x, y, width, height = box
    image = image.copy()
    image.thumbnail((width, height), Image.Resampling.LANCZOS)
    board.alpha_composite(image, (int(x + (width-image.width)/2), int(y + (height-image.height)/2)))


def comparison_board():
    board = Image.new("RGBA", (1800, 1610), CREAM)
    draw = ImageDraw.Draw(board)
    draw.text((70, 45), "AZUKI  /  MASCOTS PLAYGROUND", fill=INK, font=font(22))
    draw.text((68, 85), "A little world, carried forward.", fill=INK, font=font(63, True))
    draw.text((72, 169), "Enbito geometry, graphic characters, and imagery for the Azuki ecosystem.", fill="#6c665c", font=font(24))
    cards = [("01_enbito_king", "01  The original king", "Original Enbito mesh / new render"),
             ("03_antler_companion", "02  Antler companion", "Original mesh / sword removed"),
             ("05_branchkeeper", "03  Branchkeeper", "Vector adaptation / character art"),
             ("06_field_notes", "04  Field notes", "Vector illustration / docs companion"),
             ("07_bean_mark", "05  Bean mark", "Reduced silhouette / extension icon")]
    for i, (name, title, description) in enumerate(cards):
        col, row = i % 3, i // 3
        x, y = 66 + col * 562, 240 + row * 512
        draw.rounded_rectangle((x, y, x+544, y+492), radius=18, fill="#fffaf0")
        composite_contained(board, Image.open(ROOT / "png" / f"{name}.png").convert("RGBA"), (x+25, y+12, 494, 385))
        draw.text((x+26, y+410), title, fill=INK, font=font(27))
        draw.text((x+26, y+451), description, fill="#746c60", font=font(18))
    x, y = 1190, 752
    draw.rounded_rectangle((x, y, x+544, y+492), radius=18, fill="#202a30")
    composite_contained(board, Image.open(ROOT / "png/08_bridge_night.png").convert("RGBA"), (x+8, y+54, 528, 285))
    draw.text((x+26, y+410), "06  The crossing", fill=CREAM, font=font(27))
    draw.text((x+26, y+451), "Wide scene / README and docs headers", fill="#c4c5ad", font=font(18))
    draw.text((72, 1292), "Same world. Different scales.", fill=INK, font=font(33, True))
    composite_contained(board, Image.open(ROOT / "png/08_bridge_day.png").convert("RGBA"), (70, 1344, 1100, 202))
    draw.rounded_rectangle((1210, 1340, 1734, 1548), radius=16, fill="#202a30")
    for index, size in enumerate([16, 24, 32, 48, 96]):
        icon = Image.open(ROOT / "png/icons" / f"bean_dark_{size}.png").convert("RGBA")
        center = 1250 + index * 95
        board.alpha_composite(icon, (center-size//2, 1438-size//2))
        draw.text((center-12, 1505), str(size), fill="#c9c7b6", font=font(16))
    board.convert("RGB").save(ROOT / "previews/overview.jpg", quality=94)


def contexts_board():
    board = Image.new("RGBA", (1600, 1320), CREAM)
    draw = ImageDraw.Draw(board)
    draw.text((54, 36), "AZUKI  /  CONTEXT STUDIES", fill=INK, font=font(22))
    draw.text((52, 77), "A few places these could live.", fill=INK, font=font(48, True))
    draw.rounded_rectangle((54, 160, 1546, 620), radius=20, fill="#fffaf0")
    scene = Image.open(ROOT / "png/08_bridge_day.png").convert("RGBA").resize((840, 294), Image.Resampling.LANCZOS)
    board.alpha_composite(scene, (656, 220))
    draw.text((104, 241), "Azuki", fill=INK, font=font(86, True))
    draw.text((108, 350), "A minimalist parser generator for Rust.", fill="#666b62", font=font(26))
    draw.rounded_rectangle((108, 421, 292, 472), radius=7, fill=INK)
    draw.text((134, 433), "Read the docs", fill=CREAM, font=font(22))
    draw.text((108, 551), "README / docs masthead study", fill="#827c70", font=font(19))
    draw.rounded_rectangle((54, 654, 776, 1254), radius=20, fill="#202a30")
    composite_contained(board, Image.open(ROOT / "png/07_bean_tile.png").convert("RGBA"), (98, 704, 172, 172))
    draw.text((301, 728), "Azuki", fill=CREAM, font=font(47))
    draw.text((303, 791), "VS Code extension", fill="#b5c1b9", font=font(24))
    draw.text((103, 956), "bean mark / dark surface", fill=CREAM, font=font(27, True))
    for i, size in enumerate([16, 24, 32, 48, 128]):
        board.alpha_composite(Image.open(ROOT / "png/icons" / f"bean_dark_{size}.png").convert("RGBA"), (106+i*117, 1080-size//2))
        draw.text((107+i*117, 1175), f"{size}px", fill="#c4c5ad", font=font(18))
    draw.rounded_rectangle((810, 654, 1546, 1254), radius=20, fill="#fffaf0")
    composite_contained(board, Image.open(ROOT / "png/06_field_notes.png").convert("RGBA"), (851, 684, 654, 422))
    draw.text((865, 1130), "Room for a field guide.", fill=INK, font=font(34, True))
    draw.text((866, 1185), "Docs landing page / supporting illustration", fill="#746c60", font=font(22))
    board.convert("RGB").save(ROOT / "previews/contexts.jpg", quality=94)


def provenance_manifest():
    sources = []
    for path in sorted((ROOT / "sources/enbito").glob("*.glb")):
        sources.append({"file": str(path.relative_to(ROOT)), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "original": f"/Users/kyle/enbito/public/models/{path.name}"})
    (ROOT / "sources/provenance.json").write_text(json.dumps({
        "source_project": "Enbito (2023)", "source_models": sources,
        "method": "Original bind-pose geometry rendered locally; vector studies drawn procedurally from the character design.",
        "preserved_traits": ["red faceted bean body", "large black eyes", "chunky black feet", "branching brown antlers", "sword, spear and banner motifs"],
        "new_elements": ["simplified vector shapes", "open book and small hands", "bridge landscape", "compact icon proportions", "day and night palettes"],
        "fonts": "Comparison boards use local system fonts; reusable SVG imagery contains no fonts or text."
    }, indent=2) + "\n")


def main():
    if not shutil.which("rsvg-convert"):
        raise SystemExit("rsvg-convert is required to export the SVG studies as PNGs.")
    for directory in ["svg", "png", "png/icons", "previews"]:
        (ROOT / directory).mkdir(parents=True, exist_ok=True)
    king = read_glb(ROOT / "sources/enbito/azuki_king.glb")
    spear = read_glb(ROOT / "sources/enbito/azuki_spear.glb")
    render(king, 1536).save(ROOT / "png/01_enbito_king.png")
    render(spear, 1536, azimuth=-34, elevation=15).save(ROOT / "png/02_enbito_spear.png")
    render(king, 1536, azimuth=-16, weapon=False).save(ROOT / "png/03_antler_companion.png")
    render(king, 1536, azimuth=-14, antlers=False, weapon=False).save(ROOT / "png/04_little_bean.png")
    export_svg("05_branchkeeper", 512, 512, branchkeeper(), "Azuki branchkeeper", "A red bean with round black eyes, black feet, and tall branching antlers.")
    export_svg("05_branchkeeper_sword", 512, 512, sword() + branchkeeper(), "Azuki branchkeeper with sword", "A graphic interpretation of the antlered Enbito king holding a short sword.")
    export_svg("06_field_notes", 800, 800, field_notes(), "Azuki field notes", "An antlered bean reading a book on a small mossy rock, with a dragonfly nearby.")
    export_svg("07_bean_mark", 512, 512, compact_mark(), "Azuki bean mark", "A simple red bean face with two short branching antlers.")
    export_svg("07_bean_mark_dark", 512, 512, compact_mark(dark=True), "Azuki bean mark for dark surfaces", "A simple red bean face with two gold branching antlers for dark backgrounds.")
    export_svg("07_bean_tile", 512, 512, f'<rect x="12" y="12" width="488" height="488" rx="112" fill="{CREAM}"/><g transform="translate(38 22) scale(.85)">{compact_mark()}</g>', "Azuki bean icon tile", "The compact antlered bean on a warm cream rounded square.")
    export_svg("07_tiny_bean", 16, 16, tiny_mark(), "Azuki tiny bean", "A red bean face simplified for sixteen and twenty-four pixel icons.", 256)
    for mode in ["day", "night"]:
        export_svg(f"08_bridge_{mode}", 1600, 560, bridge_scene(mode == "night"), f"Azuki crossing, {mode}", "Three bean companions cross a stone bridge, carrying a sword, antlers, a book, and a bean banner.", 2400)
    for dark in [False, True]:
        suffix = "_dark" if dark else ""
        for size in [16, 24, 32, 48, 64, 96, 128, 256, 512]:
            source = ROOT / "svg" / ("07_tiny_bean.svg" if size <= 24 else f"07_bean_mark{suffix}.svg")
            subprocess.run(["rsvg-convert", "--width", str(size), "--height", str(size), "--output",
                            str(ROOT / "png/icons" / f"bean{suffix}_{size}.png"), str(source)], check=True)
    comparison_board()
    contexts_board()
    provenance_manifest()
    print("Rendered 4 original-model studies, 9 vector studies, 18 icons and 2 comparison boards.")


if __name__ == "__main__":
    main()
