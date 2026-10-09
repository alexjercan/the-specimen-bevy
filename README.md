# The Specimen

A facility horror game built with Bevy.

## Quickstart

```bash
cargo run
```

## Releases

Pushing a `vX.Y.Z` tag runs `.github/workflows/release.yaml`. After all three builds succeed, it creates a GitHub Release with Linux, Windows, and HTML5 archives and notes from `CHANGELOG.md`.

For older tags that already have a successful tagged build but no GitHub Release, run **Actions > release > Run workflow** with the existing tag. This reuses its archives without rebuilding.

`.github/workflows/deploy-itch.yaml` downloads those exact published GitHub Release archives for its HTML5, Windows, and Linux channels. It remains a separate, manually approved deployment.

