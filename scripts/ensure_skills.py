#!/usr/bin/env python3
from pathlib import Path

REQUIRED_SKILLS = {
    "rust-stock-alert-bootstrap": "skills/rust-stock-alert-bootstrap/SKILL.md",
}


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    missing = []

    for skill_name, skill_path in REQUIRED_SKILLS.items():
        absolute_path = repo_root / skill_path
        if not absolute_path.exists():
            missing.append((skill_name, skill_path))

    if not missing:
        print("All required skills are present.")
        return 0

    print("Missing skills detected:")
    for skill_name, skill_path in missing:
        print(f"- {skill_name}: expected at {skill_path}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
