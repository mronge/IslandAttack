from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image
from PIL.ImageFilter import MaxFilter


ROOT = Path(__file__).resolve().parent.parent
RAW_DIR = ROOT / "output" / "imagegen" / "raw"
FINAL_DIR = ROOT / "output" / "imagegen" / "final"
FINAL_DIR.mkdir(parents=True, exist_ok=True)

SOURCE_EXTENSIONS = (".png", ".jpg", ".jpeg", ".webp")
PREVIEW_SCALE = 3
OBSOLETE_OUTPUTS = (
    "metasprite_preview.png",
    "metasprite_preview_x6.png",
    "metasprites.json",
    "nes_sprite_sheet.json",
    "nes_sprite_sheet.png",
    "nes_sprite_sheet_preview.png",
    "subtile_defs.json",
    "subtile_sheet.png",
)


@dataclass(frozen=True)
class AssetSpec:
    source: str
    draw_size: tuple[int, int]
    anchor: tuple[int, int]


ASSET_SPECS: dict[str, AssetSpec] = {
    "jeep_up": AssetSpec("jeep_up", (192, 192), (96, 96)),
    "jeep_right": AssetSpec("jeep_right", (192, 192), (96, 96)),
    "jeep_down": AssetSpec("jeep_down", (192, 192), (96, 96)),
    "jeep_left": AssetSpec("jeep_left", (192, 192), (96, 96)),
    "enemy_commando": AssetSpec("enemy_commando", (64, 64), (32, 32)),
    "enemy_rocketeer": AssetSpec("enemy_rocketeer", (64, 64), (32, 32)),
    "enemy_soldier": AssetSpec("enemy_soldier", (64, 64), (32, 32)),
    "hostage": AssetSpec("hostage", (64, 64), (32, 32)),
    "explosion": AssetSpec("explosion", (96, 96), (48, 48)),
    "grass_tile": AssetSpec("grass_tile", (128, 128), (0, 0)),
    "road_tile": AssetSpec("road_tile", (128, 128), (0, 0)),
    "water_tile": AssetSpec("water_tile", (128, 128), (0, 0)),
    "wall_tile": AssetSpec("wall_tile", (128, 128), (0, 0)),
    "cage_tile": AssetSpec("cage_tile", (128, 128), (0, 0)),
    "extraction_tile": AssetSpec("extraction_tile", (128, 128), (0, 0)),
    "bunker_turret": AssetSpec("bunker_turret", (256, 256), (128, 128)),
    "palm_tree": AssetSpec("palm_tree", (192, 256), (96, 220)),
}


def find_source_path(name: str) -> Path | None:
    for ext in SOURCE_EXTENSIONS:
        path = RAW_DIR / f"{name}{ext}"
        if path.exists():
            return path
    return None


def load_image_from_path(
    path: Path, alpha_threshold: int = 64, preserve_size: tuple[int, int] | None = None
) -> Image.Image:
    image = Image.open(path).convert("RGBA")
    if preserve_size is not None and image.size == preserve_size:
        image.putalpha(
            image.getchannel("A").point(lambda a: 255 if a >= alpha_threshold else 0)
        )
        return image

    alpha = image.getchannel("A")
    hard_alpha = alpha.point(lambda a: 255 if a >= alpha_threshold else 0)
    bbox = hard_alpha.getbbox()
    if bbox is None:
        return image
    cropped = image.crop(bbox)
    cropped.putalpha(cropped.getchannel("A").point(lambda a: 255 if a >= alpha_threshold else 0))
    return cropped


def trim_dark_matte(image: Image.Image, darkness_threshold: int = 12) -> Image.Image:
    corners = [
        image.getpixel((0, 0)),
        image.getpixel((image.width - 1, 0)),
        image.getpixel((0, image.height - 1)),
        image.getpixel((image.width - 1, image.height - 1)),
    ]
    if not all(max(r, g, b) <= darkness_threshold and a >= 250 for r, g, b, a in corners):
        return image

    bbox: tuple[int, int, int, int] | None = None
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, a = image.getpixel((x, y))
            if a < 16:
                continue
            if max(r, g, b) <= darkness_threshold:
                continue
            if bbox is None:
                bbox = (x, y, x + 1, y + 1)
            else:
                left, top, right, bottom = bbox
                bbox = (
                    min(left, x),
                    min(top, y),
                    max(right, x + 1),
                    max(bottom, y + 1),
                )

    if bbox is None:
        return image
    return image.crop(bbox)


def inset_tile_edges(image: Image.Image, inset_ratio: float = 0.025) -> Image.Image:
    inset = max(1, int(round(min(image.width, image.height) * inset_ratio)))
    if inset * 2 >= image.width or inset * 2 >= image.height:
        return image
    return image.crop((inset, inset, image.width - inset, image.height - inset))


def crop_to_aspect(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    target_aspect = size[0] / size[1]
    image_aspect = image.width / image.height

    if abs(image_aspect - target_aspect) < 0.0001:
        return image

    if image_aspect > target_aspect:
        new_width = max(1, int(round(image.height * target_aspect)))
        left = max(0, (image.width - new_width) // 2)
        return image.crop((left, 0, left + new_width, image.height))

    new_height = max(1, int(round(image.width / target_aspect)))
    top = max(0, (image.height - new_height) // 2)
    return image.crop((0, top, image.width, top + new_height))


def load_source_image(name: str, target_size: tuple[int, int]) -> Image.Image:
    path = find_source_path(name)
    if path is not None:
        image = load_image_from_path(path, preserve_size=target_size)
        if name.endswith("_tile"):
            image = trim_dark_matte(image)
            image = inset_tile_edges(image)
        return image

    if name == "bunker_turret":
        fallback = find_source_path("wall_tile")
        if fallback is not None:
            return load_image_from_path(fallback, preserve_size=target_size)

    if name == "palm_tree":
        fallback = find_source_path("grass_tile")
        if fallback is not None:
            return load_image_from_path(fallback, preserve_size=target_size)

    if name in {"enemy_commando", "enemy_rocketeer"}:
        fallback = find_source_path("enemy_soldier")
        if fallback is not None:
            return load_image_from_path(fallback, preserve_size=target_size)

    if name.startswith("jeep_"):
        base_path = find_source_path("jeep_up")
        if base_path is None:
            raise FileNotFoundError("missing raw source: expected jeep_up.[png|jpg|jpeg|webp]")
        base = load_image_from_path(base_path, preserve_size=target_size)
        if name == "jeep_right":
            return base.transpose(Image.Transpose.ROTATE_270)
        if name == "jeep_down":
            return base.transpose(Image.Transpose.ROTATE_180)
        if name == "jeep_left":
            return base.transpose(Image.Transpose.ROTATE_90)
        if name == "jeep_up":
            return base

    raise FileNotFoundError(f"missing raw source for {name}")


def fit_to_size(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    if image.size == size:
        return image.copy()

    scale = min(size[0] / image.width, size[1] / image.height)
    resized = image.resize(
        (
            max(1, int(round(image.width * scale))),
            max(1, int(round(image.height * scale))),
        ),
        Image.Resampling.NEAREST,
    )
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    offset = (
        (size[0] - resized.width) // 2,
        (size[1] - resized.height) // 2,
    )
    canvas.paste(resized, offset, resized)
    return canvas


def fit_tile_to_size(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    image = crop_to_aspect(image, size)
    if image.size == size:
        return image.copy()
    return image.resize(size, Image.Resampling.NEAREST)


def cleanup_jeep_image(image: Image.Image) -> Image.Image:
    rgba = image.convert("RGBA")
    core = Image.new("L", rgba.size, 0)
    core_pixels = core.load()

    for y in range(rgba.height):
        for x in range(rgba.width):
            r, g, b, a = rgba.getpixel((x, y))
            if a == 0:
                continue
            brightness = r + g + b
            if brightness >= 150 or (g > r + 12 and g > b + 12):
                core_pixels[x, y] = 255

    influence = core
    for _ in range(4):
        influence = influence.filter(MaxFilter(5))

    cleaned = rgba.copy()
    pixels = cleaned.load()
    for y in range(cleaned.height):
        for x in range(cleaned.width):
            r, g, b, a = pixels[x, y]
            if a == 0:
                continue
            if influence.getpixel((x, y)) == 0:
                pixels[x, y] = (0, 0, 0, 0)

    return cleaned


def build_variants(base: Image.Image) -> list[Image.Image]:
    return [
        base,
        base.transpose(Image.Transpose.FLIP_LEFT_RIGHT),
        base.transpose(Image.Transpose.FLIP_TOP_BOTTOM),
        base.transpose(Image.Transpose.ROTATE_180),
    ]


def save(name: str, image: Image.Image) -> str:
    path = FINAL_DIR / f"{name}.png"
    image.save(path)
    return str(path.relative_to(ROOT))


def manifest_entry(file: str, draw_size: tuple[int, int], anchor: tuple[int, int]) -> dict[str, float | str]:
    return {
        "file": file,
        "draw_width": draw_size[0],
        "draw_height": draw_size[1],
        "anchor_x": anchor[0],
        "anchor_y": anchor[1],
    }


def export_assets() -> dict[str, dict[str, float | str]]:
    manifest: dict[str, dict[str, float | str]] = {}

    for name, spec in ASSET_SPECS.items():
        if name.endswith("_tile"):
            image = fit_tile_to_size(
                load_source_image(spec.source, spec.draw_size), spec.draw_size
            )
        else:
            image = fit_to_size(load_source_image(spec.source, spec.draw_size), spec.draw_size)
        if name.startswith("jeep_"):
            image = cleanup_jeep_image(image)
        manifest[name] = manifest_entry(save(name, image), spec.draw_size, spec.anchor)

    grass = fit_tile_to_size(load_source_image("grass_tile", ASSET_SPECS["grass_tile"].draw_size), ASSET_SPECS["grass_tile"].draw_size)
    road = fit_tile_to_size(load_source_image("road_tile", ASSET_SPECS["road_tile"].draw_size), ASSET_SPECS["road_tile"].draw_size)
    water = fit_tile_to_size(load_source_image("water_tile", ASSET_SPECS["water_tile"].draw_size), ASSET_SPECS["water_tile"].draw_size)
    wall = fit_tile_to_size(load_source_image("wall_tile", ASSET_SPECS["wall_tile"].draw_size), ASSET_SPECS["wall_tile"].draw_size)

    for idx, image in enumerate(build_variants(grass)):
        manifest[f"ground_{idx}"] = manifest_entry(save(f"ground_{idx}", image), ASSET_SPECS["grass_tile"].draw_size, (0, 0))
    for idx, image in enumerate(build_variants(road)):
        manifest[f"road_{idx}"] = manifest_entry(save(f"road_{idx}", image), ASSET_SPECS["road_tile"].draw_size, (0, 0))
    for idx, image in enumerate(build_variants(water)[:2]):
        manifest[f"water_{idx}"] = manifest_entry(save(f"water_{idx}", image), ASSET_SPECS["water_tile"].draw_size, (0, 0))
    for idx, image in enumerate(build_variants(wall)[:2]):
        manifest[f"wall_{idx}"] = manifest_entry(save(f"wall_{idx}", image), ASSET_SPECS["wall_tile"].draw_size, (0, 0))

    return manifest


def build_preview(manifest: dict[str, dict[str, float | str]]) -> None:
    preview_names = [
        "jeep_up",
        "jeep_right",
        "jeep_down",
        "jeep_left",
        "enemy_commando",
        "enemy_rocketeer",
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
        "bunker_turret",
        "palm_tree",
    ]
    total_width = sum(int(manifest[name]["draw_width"]) for name in preview_names) + (len(preview_names) - 1) * 24
    total_height = max(int(manifest[name]["draw_height"]) for name in preview_names)
    preview = Image.new("RGBA", (total_width, total_height), (0, 0, 0, 0))

    cursor_x = 0
    for name in preview_names:
        image = Image.open(ROOT / str(manifest[name]["file"])).convert("RGBA")
        y = (total_height - image.height) // 2
        preview.paste(image, (cursor_x, y), image)
        cursor_x += image.width + 24

    preview.save(FINAL_DIR / "asset_preview.png")
    preview.resize(
        (preview.width * PREVIEW_SCALE, preview.height * PREVIEW_SCALE),
        Image.Resampling.NEAREST,
    ).save(FINAL_DIR / "asset_preview_x3.png")


def cleanup_obsolete_outputs() -> None:
    for name in OBSOLETE_OUTPUTS:
        path = FINAL_DIR / name
        if path.exists():
            path.unlink()


def main() -> None:
    cleanup_obsolete_outputs()
    manifest = export_assets()
    (FINAL_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True))
    build_preview(manifest)


if __name__ == "__main__":
    main()
