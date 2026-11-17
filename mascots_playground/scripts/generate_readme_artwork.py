"""Render Echo's full walking pose for the repository README's theme sources.

Run: python3 mascots_playground/scripts/generate_readme_artwork.py
Requires NumPy and Pillow. The light source is transparent; the dark source
uses the identical foreground and framing on a rounded cream background.
"""

import json

from PIL import Image, ImageDraw

from generate import CREAM, INK, font
from generate_poses import pose_king
from render_models import ROOT, read_glb, render


def make_artwork(config):
    poses = json.loads((ROOT / "sources/king_poses.json").read_text())
    settings = next(p for p in poses["poses"] if p["id"] == config["pose"])
    pieces = pose_king(read_glb(ROOT / "sources" / poses["source"]), settings)
    light = render(pieces, size=config["size"], azimuth=config["camera"][0],
                   elevation=config["camera"][1])
    dark = Image.new("RGBA", light.size)
    draw = ImageDraw.Draw(dark)
    draw.rounded_rectangle((0, 0, dark.width-1, dark.height-1),
                           radius=config["dark_corner_radius"], fill=config["dark_background"])
    dark.alpha_composite(light)
    return light, dark


def preview_board(light, dark, display_size):
    board = Image.new("RGB", (1280, 555), CREAM)
    draw = ImageDraw.Draw(board)
    draw.text((32, 24), "AZUKI  /  README ARTWORK", fill=INK, font=font(19))
    draw.text((30, 58), "One pose, two themes.", fill=INK, font=font(40, True))
    for index, (image, background, foreground, label) in enumerate([
        (light, "#ffffff", INK, "Light / transparent"),
        (dark, "#0d1117", CREAM, "Dark / cream panel")
    ]):
        x, y = 24+index*630, 134
        draw.rectangle((x, y, x+602, y+396), fill=background)
        draw.text((x+22, y+19), label, fill=foreground, font=font(21))
        figure = image.resize((display_size, display_size), Image.Resampling.LANCZOS)
        board.paste(figure, (x+(602-display_size)//2, y+78), figure)
    board.save(ROOT / "previews/readme_themes.jpg", quality=95)


def main():
    config = json.loads((ROOT / "sources/readme_artwork.json").read_text())
    light, dark = make_artwork(config)
    destination = ROOT.parent / "images"
    destination.mkdir(parents=True, exist_ok=True)
    light.save(destination / "echo-light.png", optimize=True)
    dark.save(destination / "echo-dark.png", optimize=True)
    preview_board(light, dark, config["display_size"])
    print(f"Generated {destination / 'echo-light.png'} and {destination / 'echo-dark.png'}.")


if __name__ == "__main__":
    main()
