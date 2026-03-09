#!/usr/bin/env python3
"""Bump the patch version of all workspace crates that depend on changed
dependencies detected from git diffs of all Cargo.toml files.

The script parses git diffs to find:
  - Changed dependencies in the root [workspace.dependencies] section
  - Workspace crates whose own Cargo.toml was modified (version bumped,
    dependencies added/removed, features changed, etc.)

Then bumps the patch version of every workspace crate that uses those
changed dependencies or depends on the modified crates.

Usage:
    ./scripts/bump-dependents.py [<git-ref>]
    ./scripts/bump-dependents.py --recursive [<git-ref>]
    ./scripts/bump-dependents.py --dry-run [<git-ref>]

Options:
    --recursive, -r     Transitively bump all downstream dependents
    --ignore, -i        Crate names to exclude from bumping
    --dry-run, -n       Print what would be bumped without modifying files

Examples:
    # Detect changed deps vs HEAD and bump dependents
    ./scripts/bump-dependents.py

    # Detect changed deps vs the main branch
    ./scripts/bump-dependents.py main

    # Recursive bump of the entire downstream chain
    ./scripts/bump-dependents.py -r

    # Ignore specific crates
    ./scripts/bump-dependents.py --ignore hoprd hoprd-api

    # Preview changes without modifying files
    ./scripts/bump-dependents.py -n
"""

import argparse
import json
import os
import re
import subprocess
import sys
from collections import defaultdict


def get_workspace_root():
    """Find the workspace root by looking for the directory containing this script."""
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def detect_changed_deps_from_diff(workspace_root, git_ref, workspace_packages):
    """Parse git diffs of all Cargo.toml files to find changed dependency names.

    For the root Cargo.toml, looks at added lines that match dependency patterns
    in [workspace.dependencies] (e.g. `crate = "version"` or `crate = { ... }`).

    For member crate Cargo.toml files, detects any modification and maps it
    back to the crate's package name.

    Args:
        workspace_root: path to the workspace root
        git_ref: git ref to diff against
        workspace_packages: dict mapping pkg_name -> (manifest_path, version)

    Returns:
        set of dependency/crate names that were changed
    """
    # Build reverse lookup: manifest_path -> package_name
    manifest_to_pkg = {}
    for pkg_name, (manifest_path, _version) in workspace_packages.items():
        manifest_to_pkg[manifest_path] = pkg_name

    cmd = ["git", "diff", "--no-color", "-U0", git_ref, "--", "Cargo.toml"]
    for manifest_path in manifest_to_pkg:
        rel = os.path.relpath(manifest_path, workspace_root)
        cmd.append(rel)

    result = subprocess.run(cmd, capture_output=True, text=True, cwd=workspace_root)
    if result.returncode != 0:
        print(f"Error running git diff: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    diff_output = result.stdout
    if not diff_output.strip():
        return set()

    changed = set()

    # Split the diff into per-file sections
    file_diffs = re.split(r"^diff --git ", diff_output, flags=re.MULTILINE)

    for file_diff in file_diffs:
        if not file_diff.strip():
            continue

        # Extract the file path from the diff header: "a/path b/path"
        header_match = re.match(r"a/(\S+)\s+b/(\S+)", file_diff)
        if not header_match:
            continue
        file_path = header_match.group(2)

        if file_path == "Cargo.toml":
            # Root Cargo.toml: extract changed dependency names from added lines
            dep_pattern = re.compile(r'^\+\s*([a-zA-Z0-9_-]+)\s*=\s*(?:"|\'|\{)', re.MULTILINE)
            for match in dep_pattern.finditer(file_diff):
                name = match.group(1)
                if not name.startswith(("+++", "---", "diff", "index")):
                    changed.add(name)
        else:
            # Member crate Cargo.toml: map to its package name
            abs_path = os.path.join(workspace_root, file_path)
            pkg_name = manifest_to_pkg.get(abs_path)
            if pkg_name:
                changed.add(pkg_name)

    return changed


def get_cargo_metadata(workspace_root):
    """Run cargo metadata and return parsed JSON."""
    result = subprocess.run(
        ["cargo", "metadata", "--format-version=1", "--no-deps"],
        capture_output=True,
        text=True,
        cwd=workspace_root,
    )
    if result.returncode != 0:
        print(f"Error running cargo metadata: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return json.loads(result.stdout)


def build_dependency_graph(metadata):
    """Build a mapping: crate_name -> set of workspace crates that depend on it.

    Returns:
        dependents: dict mapping dep_name -> set of (pkg_name, manifest_path, version)
        workspace_packages: dict mapping pkg_name -> (manifest_path, version)
    """
    dependents = defaultdict(set)
    workspace_packages = {}

    for pkg in metadata["packages"]:
        name = pkg["name"]
        manifest = pkg["manifest_path"]
        version = pkg["version"]
        workspace_packages[name] = (manifest, version)

        for dep in pkg["dependencies"]:
            dep_name = dep["name"]
            dependents[dep_name].add(name)

    return dependents, workspace_packages


def bump_patch_version(version):
    """Bump the patch component of a semver version string.

    Handles pre-release suffixes by dropping them and bumping patch.
    Examples:
        "1.2.3"      -> "1.2.4"
        "4.0.1-rc.1" -> "4.0.2"
        "0.10.0"     -> "0.10.1"
    """
    # Strip pre-release and build metadata
    base = re.split(r"[-+]", version, maxsplit=1)[0]
    parts = base.split(".")
    if len(parts) != 3:
        print(f"Warning: cannot parse version '{version}', skipping", file=sys.stderr)
        return None
    major, minor, patch = parts
    new_patch = int(patch) + 1
    return f"{major}.{minor}.{new_patch}"


def bump_version_in_manifest(manifest_path, old_version, new_version, dry_run=False):
    """Replace the version field in a Cargo.toml file."""
    with open(manifest_path, "r") as f:
        content = f.read()

    # Match the version line in the [package] section
    # We look for `version = "X.Y.Z"` or `version = "X.Y.Z-pre"`
    # that appears near the top of the file (in the [package] section)
    old_pattern = f'version = "{re.escape(old_version)}"'
    new_line = f'version = "{new_version}"'

    # Only replace the first occurrence (the [package] version, not dependency versions)
    new_content, count = re.subn(old_pattern, new_line, content, count=1)

    if count == 0:
        print(
            f'  Warning: could not find version = "{old_version}" in {manifest_path}',
            file=sys.stderr,
        )
        return False

    if not dry_run:
        with open(manifest_path, "w") as f:
            f.write(new_content)

    return True


def find_crates_to_bump(target_crates, dependents, workspace_packages, recursive=False):
    """Find all workspace crates that need version bumps.

    Args:
        target_crates: set of crate names that were updated
        dependents: dict mapping dep_name -> set of dependent pkg names
        workspace_packages: dict mapping pkg_name -> (manifest_path, version)
        recursive: if True, transitively follow the dependency chain

    Returns:
        set of workspace package names to bump
    """
    to_bump = set()
    queue = list(target_crates)
    visited = set()

    while queue:
        crate = queue.pop(0)
        if crate in visited:
            continue
        visited.add(crate)

        # Find workspace crates that depend on this crate
        for dep_name in dependents.get(crate, set()):
            if dep_name in workspace_packages and dep_name not in target_crates:
                to_bump.add(dep_name)
                if recursive:
                    queue.append(dep_name)

    return to_bump


def main():
    parser = argparse.ArgumentParser(
        description="Bump patch version of workspace crates whose dependencies changed in Cargo.toml.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                            # Detect changes vs HEAD, bump dependents
  %(prog)s main                       # Detect changes vs main branch
  %(prog)s -r                         # Recursive bump of downstream chain
  %(prog)s -n                         # Dry-run preview
  %(prog)s -i hoprd hoprd-api         # Ignore specific crates
        """,
    )
    parser.add_argument(
        "git_ref",
        nargs="?",
        default="HEAD",
        metavar="GIT_REF",
        help="Git ref to diff Cargo.toml against (default: HEAD)",
    )
    parser.add_argument(
        "--recursive",
        "-r",
        action="store_true",
        help="Transitively bump all downstream dependents",
    )
    parser.add_argument(
        "--ignore",
        "-i",
        nargs="+",
        default=[],
        metavar="CRATE",
        help="Crate names to exclude from bumping",
    )
    parser.add_argument(
        "--dry-run",
        "-n",
        action="store_true",
        help="Show what would be bumped without modifying files",
    )

    args = parser.parse_args()

    workspace_root = get_workspace_root()

    metadata = get_cargo_metadata(workspace_root)
    dependents, workspace_packages = build_dependency_graph(metadata)

    target_crates = detect_changed_deps_from_diff(workspace_root, args.git_ref, workspace_packages)
    if not target_crates:
        print("No changes detected in any Cargo.toml files.")
        return

    print(f"Detected changed dependencies: {', '.join(sorted(target_crates))}\n")

    # Validate that detected deps exist in the dependency graph
    all_known_deps = set(dependents.keys()) | set(workspace_packages.keys())
    unknown = target_crates - all_known_deps
    if unknown:
        print(
            f"Warning: these crates are not found in the workspace dependency graph: {', '.join(sorted(unknown))}",
            file=sys.stderr,
        )

    # Find crates that need bumping
    to_bump = find_crates_to_bump(target_crates, dependents, workspace_packages, recursive=args.recursive)

    # Apply ignore list
    ignored = set(args.ignore)
    if ignored:
        removed = to_bump & ignored
        if removed:
            print(f"Ignoring: {', '.join(sorted(removed))}\n")
        to_bump -= ignored

    if not to_bump:
        print("No workspace crates to bump.")
        return

    # Sort for deterministic output
    to_bump_sorted = sorted(to_bump)

    if args.dry_run:
        print("Dry run — the following crates would be bumped:\n")

    bumped = 0
    for pkg_name in to_bump_sorted:
        manifest_path, old_version = workspace_packages[pkg_name]
        new_version = bump_patch_version(old_version)
        if new_version is None:
            continue

        rel_path = os.path.relpath(manifest_path, workspace_root)
        prefix = "[dry-run] " if args.dry_run else ""
        print(f"{prefix}{pkg_name}: {old_version} -> {new_version}  ({rel_path})")

        if bump_version_in_manifest(manifest_path, old_version, new_version, dry_run=args.dry_run):
            bumped += 1

    print(f"\n{'Would bump' if args.dry_run else 'Bumped'} {bumped} crate(s).")


if __name__ == "__main__":
    main()
