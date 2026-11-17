"""Small, offline orthographic renderer for the original Enbito model geometry.

Requires Python 3, NumPy and Pillow. No game runtime or Blender installation.
The copied GLBs use model-space bind-pose geometry. Animation is not evaluated.
"""

from pathlib import Path
import argparse
import json
import math
import struct

import numpy as np
from PIL import Image


ROOT = Path(__file__).resolve().parents[1]


def read_glb(path):
    data = path.read_bytes()
    length = struct.unpack_from("<I", data, 12)[0]
    gltf = json.loads(data[20:20 + length])
    binary = data[28 + length:]

    def accessor(index):
        item = gltf["accessors"][index]
        view = gltf["bufferViews"][item["bufferView"]]
        dtype = {5120: "i1", 5121: "u1", 5122: "<i2", 5123: "<u2",
                 5125: "<u4", 5126: "<f4"}[item["componentType"]]
        count = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}[item["type"]]
        offset = view.get("byteOffset", 0) + item.get("byteOffset", 0)
        stride = view.get("byteStride", count * np.dtype(dtype).itemsize)
        return np.ndarray((item["count"], count), dtype=dtype, buffer=binary,
                          offset=offset, strides=(stride, np.dtype(dtype).itemsize)).copy()

    pieces = []
    for node in gltf["nodes"]:
        if "mesh" not in node:
            continue
        for primitive in gltf["meshes"][node["mesh"]]["primitives"]:
            vertices = accessor(primitive["attributes"]["POSITION"])
            indices = accessor(primitive["indices"]).reshape(-1, 3)
            material = gltf["materials"][primitive["material"]]
            color = material["pbrMetallicRoughness"].get("baseColorFactor", [1, 1, 1, 1])[:3]
            pieces.append({"name": node.get("name", ""), "material": material["name"],
                           "vertices": vertices, "triangles": indices,
                           "color": np.array(color)})
    return pieces


def normalize(vector):
    return vector / np.maximum(np.linalg.norm(vector, axis=-1, keepdims=True), 1e-10)


def render(pieces, size=1024, azimuth=-22, elevation=12, antlers=True,
           weapon=True, palette=None):
    """Return a transparent RGBA image. Camera faces the models' negative Z side."""
    selected = [p for p in pieces if (weapon or p["name"] != "Spear")
                and (antlers or p["material"] != "Brown")]
    yaw, pitch = math.radians(azimuth), math.radians(elevation)
    toward_camera = np.array([math.sin(yaw) * math.cos(pitch), math.sin(pitch),
                              -math.cos(yaw) * math.cos(pitch)])
    right = normalize(np.cross(np.array([0., 1., 0.]), toward_camera))
    up = normalize(np.cross(toward_camera, right))
    camera = np.stack([right, up, toward_camera])
    all_projected = np.concatenate([p["vertices"] @ camera.T for p in selected])
    low, high = all_projected.min(axis=0), all_projected.max(axis=0)
    scale = size * .81 / max((high - low)[:2])
    center = (low + high) / 2
    ss = 2
    width = height = size * ss
    pixels = np.zeros((height, width, 4), dtype=np.uint8)
    depth = np.full((height, width), -np.inf)
    key = normalize(np.array([-3., 6., -5.]))
    fill = normalize(np.array([4., 2., 1.]))

    for piece in selected:
        world = piece["vertices"]
        projected = (world @ camera.T - center) * scale * ss
        projected[:, 0] += width / 2
        projected[:, 1] = height / 2 - projected[:, 1]
        base = piece["color"] if not palette else np.array(palette.get(piece["material"], piece["color"]))
        for triangle in piece["triangles"]:
            a, b, c = projected[triangle]
            normal = normalize(np.cross(world[triangle[1]] - world[triangle[0]],
                                        world[triangle[2]] - world[triangle[0]]))
            if normal @ toward_camera <= 0:
                continue
            minimum = np.maximum(np.floor(np.minimum(np.minimum(a[:2], b[:2]), c[:2])).astype(int), 0)
            maximum = np.minimum(np.ceil(np.maximum(np.maximum(a[:2], b[:2]), c[:2])).astype(int), [width-1, height-1])
            if np.any(minimum > maximum):
                continue
            x0, y0 = minimum
            x1, y1 = maximum
            yy, xx = np.mgrid[y0:y1+1, x0:x1+1]
            xx = xx + .5
            yy = yy + .5
            denominator = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1])
            if abs(denominator) < 1e-9:
                continue
            w0 = ((b[1] - c[1]) * (xx - c[0]) + (c[0] - b[0]) * (yy - c[1])) / denominator
            w1 = ((c[1] - a[1]) * (xx - c[0]) + (a[0] - c[0]) * (yy - c[1])) / denominator
            w2 = 1 - w0 - w1
            z = w0 * a[2] + w1 * b[2] + w2 * c[2]
            tile_depth = depth[y0:y1+1, x0:x1+1]
            mask = (w0 >= -1e-7) & (w1 >= -1e-7) & (w2 >= -1e-7) & (z > tile_depth)
            light = .64 + .66 * max(0, normal @ key) + .17 * max(0, normal @ fill)
            linear = base * light
            if piece["material"] == "Black":
                linear = np.array([.009, .012, .017]) * (.5 + light)
            srgb = np.where(linear <= .0031308, 12.92 * linear,
                            1.055 * np.maximum(linear, 0) ** (1 / 2.4) - .055)
            color = np.r_[np.clip(srgb * 255, 0, 255).astype(np.uint8), 255]
            pixels[y0:y1+1, x0:x1+1][mask] = color
            tile_depth[mask] = z[mask]
    return Image.fromarray(pixels).resize((size, size), Image.Resampling.LANCZOS)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--size", type=int, default=1024)
    args = parser.parse_args()
    king = read_glb(ROOT / "sources/enbito/azuki_king.glb")
    spear = read_glb(ROOT / "sources/enbito/azuki_spear.glb")
    render(king, args.size).save(ROOT / "png/01_enbito_king.png")
    render(spear, args.size, azimuth=-34, elevation=15).save(ROOT / "png/02_enbito_spear.png")
    render(king, args.size, azimuth=-16, weapon=False).save(ROOT / "png/03_antler_companion.png")
    render(king, args.size, azimuth=-14, antlers=False, weapon=False).save(ROOT / "png/04_little_bean.png")


if __name__ == "__main__":
    main()
