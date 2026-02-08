#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.5.0"
    exit 1
fi

NEW="$1"

# Validate semver-ish format
if ! [[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: version must be in X.Y.Z format (got '$NEW')"
    exit 1
fi

cd "$(dirname "$0")"

# Discover the current version from the core Cargo.toml (single source of truth)
OLD=$(sed -n 's/^version = "\(.*\)"/\1/p' crates/ofd-validator-core/Cargo.toml)
if [ -z "$OLD" ]; then
    echo "Error: could not detect current version"
    exit 1
fi

if [ "$OLD" = "$NEW" ]; then
    echo "Already at $NEW — nothing to do."
    exit 0
fi

echo "Bumping $OLD -> $NEW"

# --- Cargo.toml files ---
for f in crates/ofd-validator-core/Cargo.toml \
         crates/ofd-validator-python/Cargo.toml \
         crates/ofd-validator-js/Cargo.toml; do
    sed -i "s/^version = \"$OLD\"/version = \"$NEW\"/" "$f"
done

# --- pyproject.toml ---
sed -i "s/^version = \"$OLD\"/version = \"$NEW\"/" pyproject.toml

# --- All tracked package.json files (version field + optionalDependencies) ---
git ls-files '*.json' | grep -v node_modules | grep -v package-lock | while read -r f; do
    sed -i "s/\"$OLD\"/\"$NEW\"/g" "$f"
done

# --- Regenerate Cargo.lock ---
cargo update --workspace 2>/dev/null

echo "Done. Files updated:"
git diff --name-only
