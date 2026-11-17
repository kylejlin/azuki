"""Generate the VS Code extension's Echo icon from the Enbito model.

Run: python3 mascots_playground/scripts/generate_extension_icon.py
Requires NumPy and Pillow. The extension packages only the final 256px PNG
and its artwork notice; this source and the larger export stay in the playground.
"""

import json

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

from generate import CREAM, INK, font
from generate_poses import pose_king
from render_models import ROOT, read_glb, render


def make_icon(config):
    poses = json.loads((ROOT / "sources/king_poses.json").read_text())
    settings = next(p for p in poses["poses"] if p["id"] == config["pose"])
    pieces = read_glb(ROOT / "sources" / poses["source"])
    for piece in pieces:
        if piece["material"] == "Brown":
            pivot = np.array(config["antler_pivot"])
            piece["vertices"] = ((piece["vertices"] - pivot)
                                 * config["antler_scale"] + pivot)
    pieces = pose_king(pieces, settings)
    size = config["master_size"]
    character = render(pieces, size=size, azimuth=config["camera"][0],
                       elevation=config["camera"][1], palette=config["palette"])
    # A thin contour preserves the narrow sword and antler edges after reduction.
    silhouette = character.getchannel("A").filter(ImageFilter.MaxFilter(9))
    edge = Image.new("RGBA", character.size, tuple(config["outline"]))
    edge.putalpha(silhouette.point(lambda alpha: round(alpha * config["outline"][3] / 255)))
    image = Image.new("RGBA", character.size, config["background"])
    image.alpha_composite(edge)
    image.alpha_composite(character)
    return image.convert("RGB")


def preview_board(image):
    board = Image.new("RGB", (1440, 670), CREAM)
    draw = ImageDraw.Draw(board)
    draw.text((35, 28), "AZUKI  /  VS CODE EXTENSION ICON", fill=INK, font=font(19))
    draw.text((33, 66), "Echo, ready for the extension.", fill=INK, font=font(43, True))
    for index, (background, foreground, label) in enumerate([
        ("#ffffff", INK, "Light surface"), ("#1f1f1f", CREAM, "Dark surface")
    ]):
        x, y = 30 + index * 705, 145
        draw.rectangle((x, y, x+675, y+480), fill=background)
        draw.text((x+24, y+22), label, fill=foreground, font=font(22))
        for size, left in [(256, 28), (128, 326), (64, 509)]:
            top = y+90+(256-size)//2
            board.paste(image.resize((size, size), Image.Resampling.LANCZOS), (x+left, top))
            draw.text((x+left, y+379), f"{size} px", fill=foreground, font=font(20))
    board.save(ROOT / "previews/extension_icon.jpg", quality=95)


def main():
    config = json.loads((ROOT / "sources/extension_icon.json").read_text())
    master = make_icon(config)
    master.save(ROOT / "png/09_extension_echo.png", optimize=True)
    destination = ROOT.parent / "azuki-vscode/images/icon.png"
    destination.parent.mkdir(parents=True, exist_ok=True)
    master.resize((config["icon_size"], config["icon_size"]), Image.Resampling.LANCZOS).save(destination, optimize=True)
    preview_board(master)
    print(f"Generated {destination} and the playground master and preview.")


if __name__ == "__main__":
    main()
