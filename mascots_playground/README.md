# Mascots playground

Visual experiments for the Azuki ecosystem, inspired by the bean characters in
Enbito (2023). These are candidates to compare; none has been installed into the
extension, the project README, or a docs site.

![The six main imagery directions](previews/overview.jpg)

## Browse the experiments

| Study | What it explores | Assets |
| --- | --- | --- |
| Original king | Enbito's actual antlered king, with its sword and original materials, rendered from the copied game geometry. | [PNG](png/01_enbito_king.png) |
| Spear carrier | The original long-spear character. | [PNG](png/02_enbito_spear.png) |
| Antler companion | The original king with the sword hidden. | [PNG](png/03_antler_companion.png) |
| Little bean | The original bean, eyes, and feet with both sword and antlers hidden. | [PNG](png/04_little_bean.png) |
| Branchkeeper | A vector interpretation retaining the oversized branching antlers, faceted red body, black eyes, and chunky feet. | [SVG](svg/05_branchkeeper.svg), [PNG](png/05_branchkeeper.png), [sword variant](svg/05_branchkeeper_sword.svg) |
| Field notes | A book-reading companion, with shorter antlers, a rock, plants, and a dragonfly. The book and hands are new additions. | [SVG](svg/06_field_notes.svg), [PNG](png/06_field_notes.png) |
| Bean mark | A compact face and antlers for icons. Includes a gold-antler version for dark backgrounds and a cream tile. | [SVG](svg/07_bean_mark.svg), [dark-surface SVG](svg/07_bean_mark_dark.svg), [tile PNG](png/07_bean_tile.png) |
| The crossing | Three bean companions and a banner crossing a bridge. Wide artwork for a README or docs header, without embedded lettering. | [day SVG](svg/08_bridge_day.svg), [day PNG](png/08_bridge_day.png), [night SVG](svg/08_bridge_night.svg), [night PNG](png/08_bridge_night.png) |

![Imagery in a few possible contexts](previews/contexts.jpg)

## Use the images

- `svg/` contains editable vector originals for the graphic studies. They have no
  font, network, or external image dependencies.
- `png/` contains ready-to-use exports. Original-model studies are 1536 × 1536;
  the character vectors export at 1024 × 1024, field notes at 1600 × 1600,
  and banners at 2400 × 840. The tiny-bean reference export is 256 × 256.
- Characters and marks have transparent backgrounds. Field notes has a
  transparent background around its circular backdrop. Banners and the cream
  icon tile include their backgrounds.
- `png/icons/` has 16, 24, 32, 48, 64, 96, 128, 256, and 512 pixel PNGs.
  Use `bean_*.png` on light surfaces and `bean_dark_*.png` on dark surfaces.
  At 16 and 24 pixels, the bean deliberately loses its antlers so its face stays
  legible. The cream tile also works when the destination background is unknown.
- `previews/` contains comparison boards and illustrative placement studies,
  rather than changes to the real Azuki interfaces.

## Source and regeneration

The three GLBs in `sources/enbito/` were copied unchanged from
`/Users/kyle/enbito/public/models/`. `sources/provenance.json` records their
original paths and SHA-256 hashes, along with what was preserved or added.
The banner model is retained as a reference for the flag motif. The renders use
the original meshes in their bind poses; they do not evaluate game animations.
No diffusion or image-generation model was used.

The vector studies are drawn with paths in `scripts/generate.py`. The original
models are rendered by `scripts/render_models.py`, a small orthographic triangle
renderer. This keeps the experiments reproducible without Blender or the Enbito
game runtime. Model lighting is intentionally simple, without cast shadows.

From the repository root:

```sh
python3 mascots_playground/scripts/generate.py
```

Requires Python 3, NumPy, Pillow, and `rsvg-convert` on `PATH`. These were already
available on the machine used to make the experiments. Comparison boards use
local system fonts, with DejaVu fallbacks; those fonts are not bundled.
