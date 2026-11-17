"""Render new king poses from Enbito's original component meshes.

Run: python3 mascots_playground/scripts/generate_poses.py
Edit sources/king_poses.json to change a pose. Requires NumPy and Pillow.
Angles are degrees in X/Y/Z order, composed as Rz @ Ry @ Rx. The source
character faces negative Z. The source GLB and first-round images are preserved.
"""

import argparse
from copy import deepcopy
import json
import math

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

from generate import CREAM, INK, font
from render_models import ROOT, normalize, read_glb, render


GRIP = np.array([0.9578, 1.3045, 0.0])
BODY_PIVOT = np.array([0, 1.3, 0])
FOOT_PIVOTS = {"Leg.L": np.array([-0.686081, 0.637081, 0]),
               "Leg.R": np.array([0.686081, 0.637081, 0])}


def rotation(angles):
    x, y, z = np.radians(angles)
    cx, cy, cz = np.cos([x, y, z])
    sx, sy, sz = np.sin([x, y, z])
    rx = np.array([[1, 0, 0], [0, cx, -sx], [0, sx, cx]])
    ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    rz = np.array([[cz, -sz, 0], [sz, cz, 0], [0, 0, 1]])
    return rz @ ry @ rx


def move(vertices, pivot, angles, translation=(0, 0, 0)):
    return (vertices - pivot) @ rotation(angles).T + pivot + translation


def pose_sword(vertices, settings):
    direction = normalize(np.array(settings["direction"], dtype=float))
    wide = normalize(np.array([1., 0., 0.]) - direction * direction[0])
    normal = np.cross(wide, direction)
    roll = math.radians(settings.get("roll", 0))
    rolled_wide = wide * math.cos(roll) + normal * math.sin(roll)
    rolled_normal = np.cross(rolled_wide, direction)
    local = vertices - GRIP
    # Source sword: length runs toward -Z, width along Y, thickness along X.
    posed = (local[:, 1:2] * rolled_wide - local[:, 2:3] * direction
             - local[:, 0:1] * rolled_normal)
    return posed * settings.get("scale", 1) + settings["grip"]


def pose_king(original, settings):
    pieces = deepcopy(original)
    for piece in pieces:
        if piece["name"] == "Spear":
            piece["vertices"] = pose_sword(piece["vertices"], settings["sword"])
        elif piece["name"] in FOOT_PIVOTS:
            foot = settings["feet"][piece["name"]]
            piece["vertices"] = move(piece["vertices"], FOOT_PIVOTS[piece["name"]],
                                     foot["rotation"], foot["translation"])
        piece["vertices"] = move(piece["vertices"], BODY_PIVOT, settings["body_rotation"])
    foot_bottom = min(p["vertices"][:, 1].min() for p in pieces if p["name"] in FOOT_PIVOTS)
    lift = settings.get("airborne", 0) - foot_bottom
    for piece in pieces:
        piece["vertices"][:, 1] += lift
    if settings["sword"].get("plant_tip"):
        tip = min(p["vertices"][:, 1].min() for p in pieces if p["name"] == "Spear")
        for piece in pieces:
            if piece["name"] == "Spear":
                piece["vertices"][:, 1] += .02 - tip
    return pieces


def with_shadow(character, camera, config):
    """Add a ground cue only on the comparison sheet, keeping exports clean."""
    size = character.width
    yaw, pitch = np.radians(camera)
    toward = np.array([math.sin(yaw) * math.cos(pitch), math.sin(pitch),
                       -math.cos(yaw) * math.cos(pitch)])
    right = normalize(np.cross(np.array([0., 1., 0.]), toward))
    up = normalize(np.cross(toward, right))
    ground = -np.array(config["camera_target"])
    scale = size * .81 / config["frame_height"]
    x = size / 2 + ground @ right * scale
    y = size / 2 - ground @ up * scale
    shadow = Image.new("RGBA", character.size)
    draw = ImageDraw.Draw(shadow)
    draw.ellipse((x-size*.09, y-size*.013, x+size*.09, y+size*.013), fill=(65, 55, 40, 38))
    shadow = shadow.filter(ImageFilter.GaussianBlur(size * .011))
    shadow.alpha_composite(character)
    return shadow


def contact_sheet(entries, config, filename, title, subtitle, columns, card_size,
                  footer="Original meshes and materials. Individual PNGs have transparent backgrounds."):
    card_width, card_height = card_size
    rows = math.ceil(len(entries) / columns)
    margin, gap, header, footer_height = 48, 20, 174, 65
    board = Image.new("RGBA", (margin*2+columns*card_width+(columns-1)*gap,
                               header+rows*card_height+(rows-1)*gap+footer_height), CREAM)
    draw = ImageDraw.Draw(board)
    draw.text((margin, 29), "AZUKI  /  ORIGINAL ENBITO KING", fill=INK, font=font(19))
    draw.text((margin-2, 65), title, fill=INK, font=font(47, True))
    draw.text((margin, 127), subtitle, fill="#746c60", font=font(20))
    for index, entry in enumerate(entries):
        x = margin + index % columns * (card_width+gap)
        y = header + index // columns * (card_height+gap)
        draw.rounded_rectangle((x, y, x+card_width, y+card_height), radius=16, fill="#fffaf0")
        figure_size = card_width - 12
        character = Image.open(ROOT / entry["image"]).convert("RGBA")
        character = with_shadow(character, entry["camera"], config)
        character = character.resize((figure_size, figure_size), Image.Resampling.LANCZOS)
        board.alpha_composite(character, (x+6, y+3))
        draw.text((x+23, y+card_height-84), entry["label"], fill=INK, font=font(27))
        draw.text((x+23, y+card_height-42), entry["caption"], fill="#746c60", font=font(17))
    draw.text((margin, board.height-37), footer, fill="#746c60", font=font(17))
    board.convert("RGB").save(ROOT / "previews" / filename, quality=95)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--size", type=int, default=1536)
    args = parser.parse_args()
    config_path = ROOT / "sources/king_poses.json"
    config = json.loads(config_path.read_text())
    original = read_glb(ROOT / "sources" / config["source"])
    output_dir = ROOT / "png/poses"
    output_dir.mkdir(parents=True, exist_ok=True)
    entries = []
    for settings in config["poses"]:
        pieces = pose_king(original, settings)
        relative_path = f'png/poses/{settings["id"]}.png'
        azimuth, elevation = settings["camera"]
        render(pieces, args.size, azimuth=azimuth, elevation=elevation,
               frame_height=config["frame_height"], target=config["camera_target"]).save(ROOT / relative_path)
        entries.append({"image": relative_path, **settings})
    contact_sheet(entries, config, "king_poses.jpg", "The king loosens up.",
                  "Eight stances: sword placement, body lean, feet, and camera angle.", 4, (550, 650),
                  "Original meshes and materials. B and F try a shorter sword. Individual PNGs have transparent backgrounds.")
    angle_config = config["angle_study"]
    settings = next(p for p in config["poses"] if p["id"] == angle_config["pose"])
    pieces = pose_king(original, settings)
    angle_entries = []
    for view in angle_config["views"]:
        relative_path = f'png/poses/angle_{view["id"]}.png'
        render(pieces, args.size, azimuth=view["camera"][0], elevation=view["camera"][1],
               frame_height=config["frame_height"], target=config["camera_target"]).save(ROOT / relative_path)
        angle_entries.append({"image": relative_path, "caption": "Same easy-guard pose", **view})
    contact_sheet(angle_entries, config, "king_angles.jpg", "One pose, three views.",
                  "A closer look at the easy guard, with only the camera changing.", 3, (650, 750))
    print("Rendered 8 king poses and 3 camera studies, with two comparison sheets.")


if __name__ == "__main__":
    main()
