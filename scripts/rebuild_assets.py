from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parent.parent
FINAL_DIR = ROOT / "output" / "imagegen" / "final"
FINAL_DIR.mkdir(parents=True, exist_ok=True)

SUBTILE = 8
PREVIEW_SCALE = 6


def rgba(color: str) -> tuple[int, int, int, int]:
    color = color.lstrip("#")
    return (
        int(color[0:2], 16),
        int(color[2:4], 16),
        int(color[4:6], 16),
        255,
    )


P = {
    "clear": (0, 0, 0, 0),
    "outline": rgba("#101010"),
    "shadow": rgba("#3B2A15"),
    "sand_0": rgba("#8D4E0E"),
    "sand_1": rgba("#A96512"),
    "sand_2": rgba("#C67C1A"),
    "sand_3": rgba("#E09B31"),
    "concrete_0": rgba("#454545"),
    "concrete_1": rgba("#666666"),
    "concrete_2": rgba("#8A8A8A"),
    "concrete_3": rgba("#B2B2B2"),
    "green_0": rgba("#0A3D17"),
    "green_1": rgba("#0D672A"),
    "green_2": rgba("#1E953D"),
    "green_3": rgba("#59D85E"),
    "olive": rgba("#5C6B20"),
    "tan": rgba("#E9D48A"),
    "skin": rgba("#F1C88C"),
    "rust": rgba("#A23B1F"),
    "red": rgba("#D94A2B"),
    "orange": rgba("#EF9718"),
    "yellow": rgba("#FFF08A"),
    "blue_0": rgba("#1C3D73"),
    "blue_1": rgba("#366CB6"),
    "blue_2": rgba("#7ACDFF"),
    "white": rgba("#F5F5F5"),
}


@dataclass(frozen=True)
class MetaPart:
    tile: str
    x: int
    y: int


SUBTILES: dict[str, Image.Image] = {}
METAS: dict[str, dict[str, object]] = {}


def canvas(width: int, height: int) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGBA", (width, height), P["clear"])
    return image, ImageDraw.Draw(image)


def fill(draw: ImageDraw.ImageDraw, x0: int, y0: int, x1: int, y1: int, color: str) -> None:
    draw.rectangle((x0, y0, x1, y1), fill=P[color])


def line(draw: ImageDraw.ImageDraw, pts: list[tuple[int, int]], color: str) -> None:
    for x, y in pts:
        draw.point((x, y), fill=P[color])


def frame(draw: ImageDraw.ImageDraw, x0: int, y0: int, x1: int, y1: int, color: str) -> None:
    draw.rectangle((x0, y0, x1, y1), outline=P[color])


def add_meta_from_image(name: str, image: Image.Image) -> None:
    width, height = image.size
    parts: list[dict[str, object]] = []

    for tile_y in range(0, height, SUBTILE):
        for tile_x in range(0, width, SUBTILE):
            tile = image.crop((tile_x, tile_y, tile_x + SUBTILE, tile_y + SUBTILE))
            subtile_name = f"{name}_{tile_x // SUBTILE}_{tile_y // SUBTILE}"
            SUBTILES[subtile_name] = tile
            parts.append({"tile": subtile_name, "x": tile_x, "y": tile_y})

    METAS[name] = {"w": width, "h": height, "parts": parts}


def build_sheet() -> None:
    subtile_names = sorted(SUBTILES)
    cols = 8
    rows = (len(subtile_names) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * SUBTILE, rows * SUBTILE), P["clear"])
    defs: dict[str, dict[str, int]] = {}

    for idx, name in enumerate(subtile_names):
        x = (idx % cols) * SUBTILE
        y = (idx // cols) * SUBTILE
        sheet.paste(SUBTILES[name], (x, y), SUBTILES[name])
        defs[name] = {"x": x, "y": y, "w": SUBTILE, "h": SUBTILE}

    sheet.save(FINAL_DIR / "subtile_sheet.png")
    (FINAL_DIR / "subtile_defs.json").write_text(json.dumps(defs, indent=2))
    (FINAL_DIR / "metasprites.json").write_text(json.dumps(METAS, indent=2))


def build_preview() -> None:
    meta_names = [
        "jeep_up",
        "jeep_right",
        "jeep_down",
        "jeep_left",
        "enemy_soldier",
        "hostage",
        "explosion",
        "grass_tile",
        "road_tile",
        "water_tile",
        "wall_tile",
        "cage_tile",
        "extraction_tile",
    ]
    widths = [METAS[name]["w"] for name in meta_names]
    heights = [METAS[name]["h"] for name in meta_names]
    total_width = sum(widths) + (len(meta_names) - 1) * 8
    total_height = max(heights)
    preview = Image.new("RGBA", (total_width, total_height), P["clear"])

    cursor_x = 0
    for name in meta_names:
        meta = METAS[name]
        sprite = compose_meta(name)
        y = (total_height - meta["h"]) // 2
        preview.paste(sprite, (cursor_x, y), sprite)
        cursor_x += meta["w"] + 8

    preview.save(FINAL_DIR / "metasprite_preview.png")
    preview.resize((preview.width * PREVIEW_SCALE, preview.height * PREVIEW_SCALE), Image.Resampling.NEAREST).save(
        FINAL_DIR / "metasprite_preview_x6.png"
    )


def compose_meta(name: str) -> Image.Image:
    meta = METAS[name]
    image = Image.new("RGBA", (meta["w"], meta["h"]), P["clear"])
    for part in meta["parts"]:
        tile = SUBTILES[part["tile"]]
        image.paste(tile, (part["x"], part["y"]), tile)
    return image


def sand_tile(base_variant: int) -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 0, 0, 15, 15, "sand_2")
    patterns = [
        [(0, 0), (3, 1), (7, 1), (2, 4), (5, 6), (1, 7), (6, 9), (10, 10), (14, 14)],
        [(1, 0), (6, 1), (4, 3), (0, 5), (8, 6), (3, 9), (12, 10), (6, 14), (15, 15)],
        [(2, 0), (5, 2), (9, 1), (1, 6), (7, 7), (13, 8), (4, 11), (10, 13), (15, 14)],
        [(3, 0), (8, 2), (12, 3), (2, 7), (10, 8), (0, 10), (7, 12), (13, 13), (5, 15)],
    ]
    highlights = [
        [(4, 1), (11, 4), (8, 9), (2, 12)],
        [(5, 2), (12, 5), (9, 10), (3, 13)],
        [(6, 1), (13, 4), (10, 9), (4, 12)],
        [(7, 2), (14, 5), (11, 10), (5, 13)],
    ]
    for x, y in patterns[base_variant % len(patterns)]:
        draw.point((x, y), fill=P["sand_0"])
    for x, y in highlights[base_variant % len(highlights)]:
        draw.point((x, y), fill=P["sand_3"])
    return image


def grass_tile() -> Image.Image:
    image = sand_tile(0)
    draw = ImageDraw.Draw(image)
    tufts = [
        [(2, 2), (3, 2), (2, 3), (11, 3), (12, 3), (12, 4), (5, 10), (6, 10), (5, 11)],
        [(3, 4), (4, 4), (4, 5), (10, 9), (11, 9), (11, 10), (13, 12), (14, 12), (14, 13)],
    ]
    for tuft in tufts:
        line(draw, tuft, "green_2")
    shadows = [(4, 3), (13, 4), (6, 11), (11, 10), (14, 14)]
    line(draw, shadows, "green_0")
    return image


def road_tile() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 0, 0, 15, 15, "concrete_2")
    for x in (4, 9, 14):
        fill(draw, x, 0, min(x + 1, 15), 15, "concrete_3")
    for y in (2, 7, 12):
        fill(draw, 0, y, 15, y, "concrete_1")
    cracks = [
        [(1, 3), (2, 4), (3, 5), (2, 5)],
        [(10, 2), (11, 3), (12, 4), (11, 4)],
        [(5, 9), (6, 10), (7, 11), (6, 11)],
        [(9, 10), (10, 11), (11, 12), (10, 12)],
    ]
    for crack in cracks:
        line(draw, crack, "outline")
    return image


def water_tile() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 0, 0, 15, 15, "blue_0")
    waves = [
        [(1, 1), (2, 1), (6, 3), (7, 3), (11, 5), (12, 5), (3, 9), (4, 9), (9, 13), (10, 13)],
        [(4, 2), (5, 2), (9, 4), (10, 4), (0, 7), (1, 7), (7, 10), (8, 10), (13, 12), (14, 12)],
    ]
    for wave in waves[0]:
        draw.point(wave, fill=P["blue_2"])
    for wave in waves[1]:
        draw.point(wave, fill=P["blue_1"])
    return image


def wall_tile() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 0, 0, 15, 15, "concrete_1")
    for y in range(0, 16, 4):
        fill(draw, 0, y, 15, min(y + 1, 15), "concrete_0")
    for x in (0, 5, 10, 15):
        fill(draw, x, 0, x, 15, "concrete_3")
    bolts = [(3, 3), (8, 4), (13, 3), (2, 10), (9, 9), (14, 11)]
    line(draw, bolts, "outline")
    frame(draw, 0, 0, 15, 15, "outline")
    return image


def cage_tile() -> Image.Image:
    image = sand_tile(1)
    draw = ImageDraw.Draw(image)
    fill(draw, 1, 1, 14, 14, "concrete_0")
    fill(draw, 2, 2, 13, 13, "concrete_2")
    for x in (4, 7, 10, 13):
        fill(draw, x, 2, x, 13, "tan")
    fill(draw, 2, 5, 13, 5, "outline")
    fill(draw, 2, 10, 13, 10, "outline")
    frame(draw, 1, 1, 14, 14, "outline")
    return image


def extraction_tile() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 0, 0, 15, 15, "green_1")
    fill(draw, 2, 2, 13, 13, "sand_3")
    fill(draw, 7, 3, 8, 12, "concrete_0")
    fill(draw, 3, 7, 12, 8, "concrete_0")
    frame(draw, 1, 1, 14, 14, "outline")
    return image


def enemy_soldier() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 6, 2, 9, 4, "skin")
    fill(draw, 5, 2, 10, 3, "outline")
    fill(draw, 5, 5, 10, 10, "olive")
    fill(draw, 6, 6, 9, 7, "red")
    fill(draw, 7, 11, 8, 13, "outline")
    fill(draw, 4, 7, 4, 10, "outline")
    fill(draw, 11, 6, 12, 7, "outline")
    return image


def hostage() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 6, 2, 9, 4, "skin")
    fill(draw, 5, 2, 10, 3, "outline")
    fill(draw, 5, 5, 10, 10, "white")
    fill(draw, 6, 11, 7, 13, "blue_0")
    fill(draw, 8, 11, 9, 13, "blue_0")
    fill(draw, 4, 6, 4, 9, "outline")
    fill(draw, 11, 6, 11, 9, "outline")
    return image


def explosion() -> Image.Image:
    image, draw = canvas(16, 16)
    fill(draw, 6, 5, 9, 9, "yellow")
    fill(draw, 7, 6, 8, 8, "white")
    line(draw, [(7, 2), (8, 2), (4, 7), (11, 7), (7, 12), (8, 12)], "orange")
    line(draw, [(5, 4), (10, 4), (5, 10), (10, 10), (3, 7), (12, 7)], "red")
    return image


def jeep(direction: str) -> Image.Image:
    image, draw = canvas(32, 32)
    fill(draw, 8, 9, 23, 27, "shadow")
    fill(draw, 10, 28, 21, 29, "shadow")
    fill(draw, 7, 10, 9, 15, "outline")
    fill(draw, 22, 10, 24, 15, "outline")
    fill(draw, 7, 21, 9, 26, "outline")
    fill(draw, 22, 21, 24, 26, "outline")
    fill(draw, 10, 6, 21, 25, "green_1")
    fill(draw, 11, 7, 20, 24, "green_2")
    fill(draw, 12, 8, 19, 11, "green_3")
    fill(draw, 12, 12, 19, 16, "white")
    fill(draw, 14, 13, 17, 15, "blue_2")
    fill(draw, 11, 21, 20, 24, "rust")
    frame(draw, 10, 6, 21, 25, "outline")
    fill(draw, 9, 9, 9, 22, "outline")
    fill(draw, 22, 9, 22, 22, "outline")
    if direction == "up":
        fill(draw, 13, 3, 18, 4, "tan")
        fill(draw, 14, 2, 17, 2, "outline")
        fill(draw, 14, 28, 17, 30, "rust")
    elif direction == "down":
        fill(draw, 13, 27, 18, 28, "tan")
        fill(draw, 14, 29, 17, 29, "outline")
        fill(draw, 14, 2, 17, 4, "rust")
    elif direction == "left":
        fill(draw, 3, 13, 4, 18, "tan")
        fill(draw, 2, 14, 2, 17, "outline")
        fill(draw, 27, 14, 30, 17, "rust")
        fill(draw, 12, 8, 19, 11, "green_3")
    else:
        fill(draw, 27, 13, 28, 18, "tan")
        fill(draw, 29, 14, 29, 17, "outline")
        fill(draw, 2, 14, 5, 17, "rust")
        fill(draw, 12, 8, 19, 11, "green_3")
    return image


def export_assets() -> None:
    assets = {
        "grass_tile": grass_tile(),
        "road_tile": road_tile(),
        "water_tile": water_tile(),
        "wall_tile": wall_tile(),
        "cage_tile": cage_tile(),
        "extraction_tile": extraction_tile(),
        "enemy_soldier": enemy_soldier(),
        "hostage": hostage(),
        "explosion": explosion(),
        "jeep_up": jeep("up"),
        "jeep_right": jeep("right"),
        "jeep_down": jeep("down"),
        "jeep_left": jeep("left"),
    }

    for name, image in assets.items():
        image.save(FINAL_DIR / f"{name}.png")
        add_meta_from_image(name, image)


def main() -> None:
    export_assets()
    build_sheet()
    build_preview()


if __name__ == "__main__":
    main()
