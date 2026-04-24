from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:
    import tomli as tomllib  # type: ignore[no-redef]


def walk(
    node: object,
    path: list[str],
    cargo_toml: Path,
    allowed_crates: set[str],
    violations: list[str],
) -> None:
    if isinstance(node, dict):
        for key, value in node.items():
            if isinstance(value, dict) and "git" in value:
                package_name = value.get("package", key)
                if package_name not in allowed_crates:
                    table_path = ".".join(path + [str(key)])
                    violations.append(f"{cargo_toml}: {table_path} uses git = {value.get('git')!r}")
            walk(value, path + [str(key)], cargo_toml, allowed_crates, violations)
    elif isinstance(node, list):
        for index, value in enumerate(node):
            walk(value, path + [f"[{index}]"], cargo_toml, allowed_crates, violations)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate that Cargo dependencies do not use git sources unless allowed."
    )
    parser.add_argument(
        "--allow",
        dest="allowed_crates",
        action="append",
        default=[],
        help="Crate name allowed to use git-based dependencies. Can be repeated.",
    )
    return parser.parse_args()


def workspace_cargo_tomls(root: Path) -> list[Path]:
    root_cargo = root / "Cargo.toml"
    try:
        root_data = tomllib.loads(root_cargo.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"Failed to parse {root_cargo}: {exc}", file=sys.stderr)
        raise SystemExit(1)

    members = root_data.get("workspace", {}).get("members", [])
    cargo_files = [root_cargo]

    for member in members:
        member_cargo = root / member / "Cargo.toml"
        if member_cargo.exists():
            cargo_files.append(member_cargo)

    return cargo_files


def cli() -> int:
    args = parse_args()
    allowed_crates = set(args.allowed_crates)
    violations: list[str] = []

    for cargo_toml in workspace_cargo_tomls(Path(".")):
        try:
            data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"Failed to parse {cargo_toml}: {exc}", file=sys.stderr)
            return 1

        walk(data, [], cargo_toml, allowed_crates, violations)

    if violations:
        print("Disallowed git-based dependencies found:")
        for violation in violations:
            print(f" - {violation}")
        if allowed_crates:
            allowed_list = ", ".join(sorted(allowed_crates))
            print(f"Only these crates may use git-based dependency declarations: {allowed_list}.")
        else:
            print("No crates are allowed to use git-based dependency declarations.")
        return 1

    print("No disallowed git-based dependencies found.")
    return 0


if __name__ == "__main__":
    raise SystemExit(cli())
