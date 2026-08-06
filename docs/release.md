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

- [ ] **The publish GitHub App is installed on `Falconiere/git-better`** with
  **Contents: read and write** + **Pull requests: read and write**. This is the
  same App as the tap push below (`HOMEBREW_APP_ID` +
  `HOMEBREW_APP_PRIVATE_KEY`), just installed on a second repo — one App does
  both jobs. A tag pushed with the default `GITHUB_TOKEN` does **not** trigger
  downstream workflows, so the bot must push the tag with its own token or
  `release.yml` never fires; both release-plz jobs mint a short-lived
  installation token, and unlike a PAT it cannot silently expire. The mint steps
  pass no `owner`/`repositories`, so each token is scoped to this repo alone even
  though the App can also reach the tap.

  Infisical syncs both secrets onto every repo that needs them, so nothing has to
  be pasted by hand — verifying they exist is a separate matter from the App
  being installed here:
  ```bash
  gh secret list --repo Falconiere/git-better | grep -E 'HOMEBREW_APP_ID|HOMEBREW_APP_PRIVATE_KEY'
  ```
  If they are missing, add them to the Infisical project rather than setting them
  per-repo. Installing the App is manual: GitHub → Settings → Developer settings
  → GitHub Apps → the App → Install App, then add `git-better` alongside
  `homebrew-tap` and grant the two permissions above. If the jobs fail with
  `403 Resource not accessible`, that install is what is missing (or a permission
  is read-only) — the secrets existing is not sufficient, since the sync is
  repo-wide while the install is per-repo.

  **Blast radius, accepted knowingly.** A GitHub App has one installation per
  account, with a single permission grant shared by every repo selected in it. So
  adding `Pull requests: read and write` for release-plz also exposes that
  permission on `homebrew-tap` and on comemory, which need only `Contents`. And
  one leaked private key now reaches this repo's tags and PRs, the tap, and
  comemory's formula publish, where the previous PAT-plus-separate-App split
  contained each. Chosen anyway: this App's key material is known-good and
  already synced, whereas a second App means a second key to install, sync and
  rotate. Revisit if the tap ever leaves this account.
- [ ] **The same App is also installed on `Falconiere/homebrew-tap`** — already
  true; it is the App that publishes comemory's formula, and the tap is comemory's
  tap. `Contents: read and write` is all the tap push itself needs, though see the
  note above for what the shared installation actually grants it. The
  `publish-homebrew-formula` job mints its own token, scoped to the tap via
  explicit `owner` + `repositories` inputs since it has to cross repos. No
  expiring PAT to rotate.
- [ ] **Enable the bot.** Both release-plz jobs are gated behind a repo
  **variable** so merging the workflow does not start cutting releases before
  the App is installed. It must be a variable, not a secret: the `vars` context
  cannot read secrets, so a secret of the same name silently skips both jobs.
  ```bash
  gh variable set RELEASE_PLZ_ENABLED --body true --repo Falconiere/git-better
  ```
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
