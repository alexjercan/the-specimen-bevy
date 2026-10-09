# The Specimen

A facility horror game built with Bevy.

## Quickstart

```bash
cargo run -p the-specimen-bevy
```

## v0.1.0 release

See [CHANGELOG.md](CHANGELOG.md) for release notes. After committing and pushing the release changes, create and push the tag to build Linux, Windows, and HTML5 archives:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Download the three archives from the tagged **release** Actions run under **Artifacts**. The HTML5 ZIP contains the game at its root and can be served as a static site. The Linux archive requires compatible system graphics and audio libraries. The Windows ZIP includes the game assets. This workflow does not publish a GitHub Release automatically.

## Web game

The site is the game itself, built from `index.html` with Trunk. No landing page is generated. To publish it:

1. In the GitHub repository, open **Settings > Pages**. Set **Build and deployment > Source** to **GitHub Actions**.
2. Open **Actions > deploy-pages > Run workflow**, select the branch to deploy, and run it.
3. After the deployment finishes, open `https://alexjercan.github.io/the-specimen-bevy/` in a WebGPU-capable browser.

Deployments are manual only. To preview the site locally without a release build, run `nix develop --command trunk serve`.

## itch.io

The manual [deploy-itch workflow](.github/workflows/deploy-itch.yaml) downloads the HTML5, Windows, and Linux archives from a successful tagged **release** Actions run and uploads them to the `html5`, `windows`, and `linux` Butler channels. It does not rebuild the game or create an itch.io project. Run it before the release artifacts expire.

1. Create the itch.io project and keep its page in Draft while you test the game.
2. In GitHub, create an `itch-production` Environment with required reviewers. Set its secret `BUTLER_API_KEY` to an itch.io API key. Set its variables `ITCH_TARGET` to `user/project`, `BUTLER_VERSION` to a fixed Butler version, and `BUTLER_SHA256_LINUX_AMD64` to the SHA-256 of that version's official `linux-amd64` archive. The archive URL is `https://broth.itch.zone/butler/linux-amd64/<version>/archive/default`.
3. After the **release** workflow succeeds for a pushed version tag, open **Actions > deploy-itch > Run workflow** and enter the tag (such as `v0.1.0`). Approve the Environment deployment when prompted.
4. Check all three uploads on itch.io, mark the `html5` upload playable in the browser and the desktop uploads for the correct operating systems, and publish the itch.io page only after you approve it.

No itch.io upload runs on a push. Butler changes the three channels only when you manually run this workflow.
