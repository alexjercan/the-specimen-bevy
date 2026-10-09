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

Deployments are manual only. To preview the site locally without a release build, run `nix develop --command trunk serve`.

## itch.io HTML5

The manual [itch.io workflow](.github/workflows/deploy-itch.yaml) builds the tagged game and uploads `dist/` directly to the `html5` Butler channel. It does not upload native builds or create an itch.io project.

1. Create the itch.io project and keep its page in Draft while you test the game.
2. In GitHub, create an `itch-production` Environment with required reviewers. Set its secret `BUTLER_API_KEY` to an itch.io API key. Set its variables `ITCH_TARGET` to `user/project`, `BUTLER_VERSION` to a fixed Butler version, and `BUTLER_SHA256_LINUX_AMD64` to the SHA-256 of that version's official `linux-amd64` archive. The archive URL is `https://broth.itch.zone/butler/linux-amd64/<version>/archive/default`.
3. After committing the changes and pushing a version tag, open **Actions > Deploy HTML5 game to itch.io > Run workflow** and enter the tag (such as `v0.1.0`). Approve the Environment deployment when prompted.
4. Check the game on itch.io, mark the `html5` upload playable in the browser, and publish the itch.io page only after you approve it.

No itch.io upload runs on a push. Butler changes the `html5` channel only when you manually run this workflow.
