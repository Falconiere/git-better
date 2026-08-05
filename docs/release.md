# Release runbook (git-better)

> **TL;DR — releasing is a pull request you merge:**
>
> 1. Land your changes on `main` as usual (conventional-commit messages).
> 2. The **release-plz** bot opens/updates a **release PR** that bumps the
>    version and writes the `CHANGELOG.md` section from your commits.
> 3. Review the PR (sanity-check the version bump + changelog), then **merge it**.
> 4. Merging makes release-plz push the `vX.Y.Z` tag, which fires the dist
>    pipeline. Verify the workflows are green (§3).

This mirrors the release pattern of [`Falconiere/comemory`](https://github.com/Falconiere/comemory)
and pushes to the **same Homebrew tap**, `Falconiere/homebrew-tap`.

The release-plz bot (`.github/workflows/release-plz.yml`, config
`release-plz.toml`) owns **version + CHANGELOG + tag**. dist owns everything
after the tag: push `vX.Y.Z` → `release.yml` builds `aarch64-apple-darwin` →
uploads the tarball + shell installer to the GitHub Release → pushes the formula
to `Falconiere/homebrew-tap` (stable tags only). Two hand-maintained workflows
follow: `release-finalize.yml` smoke-tests the published artifact, and
`crates-io.yml` publishes the crate.

```
push to main ──> release-plz ──> [release PR] ──merge──> push vX.Y.Z tag
                                                              │
                            release.yml (gate + build + GitHub Release + Homebrew)
                                                              │
                       release-finalize.yml (smoke) + crates-io.yml (cargo publish)
```

`crates-io.yml` is the one deliberate difference from comemory, which is never
published to a registry. Everything else — release-plz config shape, the dist
pipeline, the App-token tap push, the finalize smoke test — matches.

---

## 1. One-time setup

- [ ] **`RELEASE_PLZ_TOKEN` (required).** A tag pushed with the default
  `GITHUB_TOKEN` does **not** trigger downstream workflows, so the bot must push
  the tag with its own token or `release.yml` never fires. Create a fine-grained
  PAT scoped to `Falconiere/git-better` with **Contents: read and write** +
  **Pull requests: read and write**:
  ```bash
  gh secret set RELEASE_PLZ_TOKEN --repo Falconiere/git-better   # paste the PAT
  ```
- [ ] **Enable the bot.** Both release-plz jobs are gated behind a repo variable
  so merging the workflow does not start cutting releases before the token
  exists:
  ```bash
  gh variable set RELEASE_PLZ_ENABLED --body true --repo Falconiere/git-better
  ```
- [ ] **`Falconiere/homebrew-tap` exists** — already true; it is comemory's tap.
- [ ] **The homebrew-publish GitHub App** (`APP_ID` + `APP_PRIVATE_KEY` secrets)
  is installed on `Falconiere/homebrew-tap` with **Contents: read and write**.
  This is the same App that publishes comemory's formula, so only the two
  secrets need copying onto this repo:
  ```bash
  gh secret list --repo Falconiere/git-better | grep -E 'APP_ID|APP_PRIVATE_KEY'
  ```
  The `publish-homebrew-formula` job mints a short-lived installation token from
  it — no expiring PAT to rotate.
- [ ] **`CARGO_REGISTRY_TOKEN`** for the crates.io publish. Without it,
  `crates-io.yml` logs a notice and skips rather than failing the release:
  ```bash
  gh secret set CARGO_REGISTRY_TOKEN --repo Falconiere/git-better
  ```
- [ ] **Confirm the crates.io name `git-better`** is free or already owned. This
  blocks only the publish step.

---

## 2. Cutting a release (the bot)

1. **Merge your work to `main`** with conventional-commit subjects (`feat:`,
   `fix:`, `docs:`, …). These drive both the next semver and the changelog
   buckets in `release-plz.toml`.
2. **release-plz opens/updates the release PR** (job `release-plz-pr`), bumping
   `Cargo.toml` + `Cargo.lock` and rewriting `CHANGELOG.md`. Each push to `main`
   refreshes the same PR.
3. **Review and merge it.** `feat!:`/`fix!:` → major-ish, `feat:` → minor,
   `fix:` → patch.
4. **release-plz pushes `vX.Y.Z`** (job `release-plz-release`), which fires
   `release.yml`.

### First release

`release-plz.toml` sets `git_only = true`, so the baseline comes from git tags
rather than the registry — the bot works before the crate exists on crates.io.
If no `v*` tag exists yet, tag the current version by hand once:

```bash
git tag -a v1.0.0 -m "v1.0.0" && git push origin v1.0.0
```

That fires the same pipeline; subsequent releases go through the bot.

---

## 3. Verifying a release

```bash
gh run list --repo Falconiere/git-better --workflow release.yml --limit 3
gh release view v1.0.0 --repo Falconiere/git-better
```

Expect on the release: `git-better-aarch64-apple-darwin.tar.xz` + `.sha256`,
`git-better-installer.sh`, `sha256.sum`, `source.tar.gz`. Then:

- `release-finalize.yml` must be green (tarball layout, size, checksum).
- `Falconiere/homebrew-tap` must have a new `Formula/git-better.rb` commit.
- `crates-io.yml` must be green (published, or skipped with a notice).

Install paths, once published:

```bash
brew install Falconiere/tap/git-better
cargo install git-better
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Falconiere/git-better/releases/latest/download/git-better-installer.sh | sh
```

---

## 4. Regenerating the dist workflow

`release.yml` is generated by `dist`, with **two hand-maintained blocks** marked
by `NOTE:` comments — the quality gate in `plan`, and the App-token step in
`publish-homebrew-formula`. `allow-dirty = ["ci"]` in `Cargo.toml` keeps dist
from reporting those edits as drift.

After changing `[workspace.metadata.dist]`:

```bash
dist init --yes     # or: dist generate
```

then **re-apply both `NOTE:` blocks**, since regeneration overwrites them.
Verify with `dist plan`.
