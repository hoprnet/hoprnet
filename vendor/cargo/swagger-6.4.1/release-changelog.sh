#!/usr/bin/env bash
# Updated the changelog for a new release
set -eou pipefail

version=$1
date=$(date +%Y-%m-%d)


sed -i "s/^version = \".\+\"$/version = \"$version\"/" Cargo.toml

sed -i "s/## \[Unreleased\]/## [Unreleased]\n### Changed\n\n### Added\n\n### Fixed\n\n## [$version] - $date/" CHANGELOG.md

sed -i "s#\[Unreleased\]: https://github.com/Metaswitch/swagger-rs/compare/\(.*\)...HEAD#[Unreleased]: https://github.com/Metaswitch/swagger-rs/compare/$version...HEAD\n[$version]: https://github.com/Metaswitch/swagger-rs/compare/\1...$version#" CHANGELOG.md

echo "Now, delete any empty headers from $version in CHANGELOG.md"
