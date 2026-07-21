# Homebrew Cask

This directory holds the source-of-truth Homebrew cask for the `par-particle-life` macOS build.

## Layout

```text
homebrew/
├── README.md                  # this file
└── Casks/
    └── par-particle-life.rb   # the cask
```

The cask consumed by `brew install --cask par-particle-life` is mirrored to the
tap repository `paulrobello/homebrew-par-particle-life` by the release workflow
(see below).

## Versioning and sha256

The `version` and `sha256 arm:/intel:` values in `Casks/par-particle-life.rb`
are placeholders. **Do not edit them by hand when cutting a release.** The
[`Publish Homebrew Cask (core)`][core] workflow rewrites the entire cask file
on every release, computing the real sha256 from the just-uploaded
`par-particle-life-macos-aarch64.zip` and `par-particle-life-macos-x86_64.zip`
release assets and replacing both `version` and the `sha256` pair via `sed`.
It then commits the file back to `main` and pushes the same content to the
tap repository.

The version in this file is bumped as part of normal source edits (the rest
of the repo also tracks the new version), but the canonical sha256 is
whatever the release workflow computes — that is the only value `brew` will
actually verify against.

[core]: ../.github/workflows/publish-homebrew-cask-core.yml

## How a release updates the cask

1. A new GitHub Release is published with the macOS zip assets attached.
2. `publish-homebrew-cask-core.yml` runs (invoked by
   `publish-homebrew-cask-run.yml`).
3. The workflow:
   - downloads both macOS zips,
   - computes `shasum -a 256` for each,
   - rewrites `homebrew/Casks/par-particle-life.rb` from a template with the
     new `version` and sha256 values,
   - commits and pushes to `main`, and
   - mirrors the file to `paulrobello/homebrew-par-particle-life` (the tap
     that users actually `brew tap`).

If a release ships without the workflow running (for example a manual asset
upload with no tag push), the cask in this repository will remain on the
previous version and `brew livecheck` will report the cask as outdated. Re-run
the workflow from the Actions tab to bring it back in sync.

## Manual verification

After a release, you can sanity-check the cask locally:

```bash
# Verify the cask syntax
brew audit --cask par-particle-life

# Confirm the version Homebrew sees matches the latest GitHub release
brew livecheck par-particle-life
```
