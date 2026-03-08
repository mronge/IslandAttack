from __future__ import annotations

import json
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parent.parent
RAW_DIR = ROOT / "output" / "imagegen" / "raw"
FINAL_DIR = ROOT / "output" / "imagegen" / "final"
FINAL_DIR.mkdir(parents=True, exist_ok=True)

SUBTILE = 8
PREVIEW_SCALE = 6

SUBTILES: dict[str, Image.Image] = {}
METAS: dict[str, dict[str, object]] = {}


def cleaned_crop(name: str, alpha_threshold: int = 64) -> Image.Image:
    image = Image.open(RAW_DIR / f"{name}.png").convert("RGBA")
    alpha = image.getchannel("A")
    hard_alpha = alpha.point(lambda a: 255 if a >= alpha_threshold else 0)
    bbox = hard_alpha.getbbox()
    if bbox is None:
        raise RuntimeError(f"no opaque content found in {name}")
    cropped = image.crop(bbox)
    cropped.putalpha(cropped.getchannel("A").point(lambda a: 255 if a >= alpha_threshold else 0))
    return cropped


def fit_to_size(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    return image.resize(size, Image.Resampling.NEAREST)


def build_variants(base: Image.Image) -> list[Image.Image]:
    return [
        base,
        base.transpose(Image.Transpose.FLIP_LEFT_RIGHT),
        base.transpose(Image.Transpose.FLIP_TOP_BOTTOM),
        base.transpose(Image.Transpose.ROTATE_180),
    ]


def save_meta(name: str, image: Image.Image) -> None:
    image.save(FINAL_DIR / f"{name}.png")
    width, height = image.size
    parts = []
    for y in range(0, height, SUBTILE):
        for x in range(0, width, SUBTILE):
            tile_name = f"{name}_{x // SUBTILE}_{y // SUBTILE}"
            tile = image.crop((x, y, x + SUBTILE, y + SUBTILE))
            SUBTILES[tile_name] = tile
            parts.append({"tile": tile_name, "x": x, "y": y})
    METAS[name] = {"w": width, "h": height, "parts": parts}


def compose_meta(name: str) -> Image.Image:
    meta = METAS[name]
    image = Image.new("RGBA", (meta["w"], meta["h"]), (0, 0, 0, 0))
    for part in meta["parts"]:
        tile = SUBTILES[part["tile"]]
        image.paste(tile, (part["x"], part["y"]), tile)
    return image


def export_assets() -> None:
    # Characters and effects preserve the raw model detail.
    save_meta("jeep_up", fit_to_size(cleaned_crop("jeep_up"), (32, 32)))
    save_meta("jeep_right", fit_to_size(cleaned_crop("jeep_right"), (32, 32)))
    save_meta("jeep_down", fit_to_size(cleaned_crop("jeep_down"), (32, 32)))
    save_meta("jeep_left", fit_to_size(cleaned_crop("jeep_left"), (32, 32)))
    save_meta("enemy_soldier", fit_to_size(cleaned_crop("enemy_soldier"), (16, 16)))
    save_meta("hostage", fit_to_size(cleaned_crop("hostage"), (16, 16)))
    save_meta("explosion", fit_to_size(cleaned_crop("explosion"), (16, 16)))

    # Tiles preserve detail too, and we derive simple variants from the raw tiles.
    for idx, image in enumerate(build_variants(fit_to_size(cleaned_crop("grass_tile"), (16, 16)))):
        save_meta(f"ground_{idx}", image)
    for idx, image in enumerate(build_variants(fit_to_size(cleaned_crop("road_tile"), (16, 16)))):
        save_meta(f"road_{idx}", image)
    for idx, image in enumerate(build_variants(fit_to_size(cleaned_crop("water_tile"), (16, 16)))[:2]):
        save_meta(f"water_{idx}", image)
    for idx, image in enumerate(build_variants(fit_to_size(cleaned_crop("wall_tile"), (16, 16)))[:2]):
        save_meta(f"wall_{idx}", image)

    save_meta("cage_tile", fit_to_size(cleaned_crop("cage_tile"), (16, 16)))
    save_meta("extraction_tile", fit_to_size(cleaned_crop("extraction_tile"), (16, 16)))

    # Keep larger environment pieces as higher-detail derived assets.
    wall_large = fit_to_size(cleaned_crop("wall_tile"), (32, 32))
    save_meta("bunker_turret", wall_large)

    palm = fit_to_size(cleaned_crop("grass_tile"), (32, 32))
    save_meta("palm_tree", palm)


def build_sheet() -> None:
    names = sorted(SUBTILES)
    cols = 8
    rows = (len(names) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * SUBTILE, rows * SUBTILE), (0, 0, 0, 0))
    defs: dict[str, dict[str, int]] = {}

    for idx, name in enumerate(names):
        x = (idx % cols) * SUBTILE
        y = (idx // cols) * SUBTILE
        sheet.paste(SUBTILES[name], (x, y), SUBTILES[name])
        defs[name] = {"x": x, "y": y, "w": SUBTILE, "h": SUBTILE}

    sheet.save(FINAL_DIR / "subtile_sheet.png")
    (FINAL_DIR / "subtile_defs.json").write_text(json.dumps(defs, indent=2))
    (FINAL_DIR / "metasprites.json").write_text(json.dumps(METAS, indent=2))


def build_preview() -> None:
    preview_names = [
        "jeep_up",
        "jeep_right",
        "jeep_down",
        "jeep_left",
        "enemy_soldier",
        "hostage",
        "explosion",
        "ground_0",
        "ground_1",
        "road_0",
        "road_1",
        "water_0",
        "wall_0",
        "wall_1",
        "cage_tile",
        "extraction_tile",
    ]
    total_width = sum(METAS[name]["w"] for name in preview_names) + (len(preview_names) - 1) * 8
    total_height = max(METAS[name]["h"] for name in preview_names)
    preview = Image.new("RGBA", (total_width, total_height), (0, 0, 0, 0))

    cursor_x = 0
    for name in preview_names:
        sprite = compose_meta(name)
        y = (total_height - sprite.height) // 2
        preview.paste(sprite, (cursor_x, y), sprite)
        cursor_x += sprite.width + 8

    preview.save(FINAL_DIR / "metasprite_preview.png")
    preview.resize(
        (preview.width * PREVIEW_SCALE, preview.height * PREVIEW_SCALE),
        Image.Resampling.NEAREST,
    ).save(FINAL_DIR / "metasprite_preview_x6.png")


def main() -> None:
    export_assets()
    build_sheet()
    build_preview()


if __name__ == "__main__":
    main()
