"""Compare actual native RGB pixels, excluding timing and PNG encoder metadata."""
import argparse
from pathlib import Path

EXPECTED = {f"{theme}-{scale}x.rgb": 1280 * scale * 780 * scale * 3
            for theme in ("light", "dark") for scale in (1, 2)}


def compare(left, right):
    for root in (left, right):
        if {p.name for p in root.iterdir()} != set(EXPECTED):
            raise ValueError("native inventory must contain exactly both themes at both scales")
    for name, length in EXPECTED.items():
        a, b = (left / name).read_bytes(), (right / name).read_bytes()
        if len(a) != length or len(b) != length:
            raise ValueError(f"invalid native pixel dimensions: {name}")
        if a != b:
            first = next(i for i, (x, y) in enumerate(zip(a, b)) if x != y)
            raise ValueError(f"native pixels differ at byte {first}: {name}")
    print("native pixels identical for both themes at both scales")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", required=True, type=Path)
    parser.add_argument("--right", required=True, type=Path)
    args = parser.parse_args()
    compare(args.left, args.right)
