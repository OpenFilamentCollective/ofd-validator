# Releasing

One version number drives everything: the Rust crates, the Python wheels (PyPI), and the npm
package + its per-platform binary packages. Publishing is fully automated from a `v*` tag.

## Steps

```bash
# 1. Bump every manifest to the new version (Cargo.toml x3, pyproject.toml,
#    package.json + optionalDependencies, npm/*/package.json, and the lockfiles).
bash bump.sh X.Y.Z

# 2. Commit and push to main.
git commit -am "Release vX.Y.Z"
git push origin main

# 3. Tag the release commit and push THAT tag (not --tags — see gotchas).
git tag vX.Y.Z
git push origin vX.Y.Z
```

Pushing the tag triggers both workflows:

- **`build-npm.yml`** → builds the native addon for all 5 targets, copies each `.node` into its
  `npm/<platform>/` package (`napi artifacts`), verifies none are empty, and publishes the platform
  packages **and** the main loader to npm with provenance.
- **`build-wheels.yml`** → builds + publishes the Python wheels to PyPI.

Watch them under the repo's **Actions** tab.

## Prerequisites (one-time)

- **npm:** an `NPM_TOKEN` **repository secret** (an npm *Automation* token with publish rights to the
  `@openfilamentcollective` scope). See the [npm publishing setup].
- **PyPI:** a trusted publisher (OIDC) or API token configured for `build-wheels.yml`.

## Gotchas (these bit us — don't repeat them)

- **The published version comes from the manifests at the tagged commit, not the tag name.** Always
  bump + commit *before* tagging, and tag the commit that has the bump.
- **`git push origin vX.Y.Z`, never `git push --tags`.** `--tags` resurrects stale local tags and
  triggers pointless (or wrong-version) runs.
- **The version must be new.** npm and PyPI both refuse to overwrite an existing version. The npm
  publish steps are idempotent (they skip already-published packages), so re-running a tag is safe.
- **Re-triggering after a workflow fix on the *same* version:** move the tag onto the fix
  (`git tag -f vX.Y.Z && git push -f origin vX.Y.Z`), or use **Actions → Run workflow** (dispatch runs
  from `main`). Tag runs use the workflow file *as of the tagged commit*, so a fix only on `main`
  won't apply until the tag moves.
- **Don't add a `prepublishOnly` (or other publish lifecycle) script to any package.json.** `napi
  prepublish` in such a hook re-publishes the platform packages on every `npm publish` → 403. The CI
  publishes explicitly instead (and passes `--ignore-scripts`).
- **`napi prepublish` is not used in CI** — it publishes without the auth token in scope (→ 404).
  `napi artifacts` places the binaries; the workflow publishes each package with the token set.

[npm publishing setup]: https://docs.npmjs.com/creating-and-viewing-access-tokens
