# The Specimen

A facility horror game built with Bevy.

## Quickstart

```bash
cargo run
```

## Releases

Pushing a `vX.Y.Z` tag runs `.github/workflows/release.yaml`. After all three builds succeed, it creates a GitHub Release with Linux, Windows, and HTML5 archives and notes from `CHANGELOG.md`.

The `v0.1.0` tag was pushed before this publish job existed. After its tagged build succeeds and these workflow changes are on the default branch, run **Actions > release > Run workflow** with `tag` set to `v0.1.0`. That manual path reuses the successful tagged build and creates the missing GitHub Release without rebuilding.

`.github/workflows/deploy-itch.yaml` downloads those exact published GitHub Release archives for its HTML5, Windows, and Linux channels. It remains a separate, manually approved deployment.

