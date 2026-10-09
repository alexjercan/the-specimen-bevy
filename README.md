# The Specimen

A facility horror game built with Bevy.

## Quickstart

```bash
cargo run -p the-specimen-bevy
```

## v0.1.0 release

See [CHANGELOG.md](CHANGELOG.md) for release notes. After committing and pushing the release changes, create and push the tag to build Linux and Windows archives:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Download the archives from the tagged Actions run under **Artifacts**. The Linux archive requires compatible system graphics and audio libraries. The Windows ZIP includes the game assets. This workflow does not publish a GitHub Release automatically.

## Web game

The site is the game itself, built from `index.html` with Trunk. No landing page is generated. To publish it:

1. In the GitHub repository, open **Settings > Pages**. Set **Build and deployment > Source** to **GitHub Actions**.
2. Open **Actions > Deploy game to GitHub Pages > Run workflow**, select the branch to deploy, and run it.
3. After the deployment finishes, open `https://alexjercan.github.io/the-specimen-bevy/` in a WebGPU-capable browser.

Deployments are manual only. To build the same site locally, run `nix develop --command trunk build --release` and inspect `dist/`.
