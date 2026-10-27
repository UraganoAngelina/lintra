"""Compare actual native RGB pixels, excluding timing and PNG encoder metadata."""
import argparse
from pathlib import Path

EXPECTED = {"results-1x.rgb": 1280 * 780 * 3, "results-2x.rgb": 2560 * 1560 * 3}


def compare(left, right):
    for root in (left, right):
        if {p.name for p in root.iterdir()} != set(EXPECTED):
            raise ValueError("native visual inventory must contain both scales only")
    for name, length in EXPECTED.items():
        a, b = (left / name).read_bytes(), (right / name).read_bytes()
        if len(a) != length or len(b) != length:
            raise ValueError(f"invalid native pixel dimensions: {name}")
        if a != b:
            first = next(i for i, (x, y) in enumerate(zip(a, b)) if x != y)
            raise ValueError(f"native pixels differ at byte {first}: {name}")
    print("native Linux/Windows pixels identical at both scales")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", required=True, type=Path)
    parser.add_argument("--right", required=True, type=Path)
    args = parser.parse_args()
    compare(args.left, args.right)
