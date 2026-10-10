# Achievements

The game tracks seven achievements. Open **Achievements** below Credits on the main menu to see the locked/unlocked status, criteria, and icon for each one. New unlocks show a short in-game toast. Progress survives new runs and game restarts. The main-menu build saves progress; automated headless runs do not write player data.

| Achievement | Steam API name | Unlock condition |
| --- | --- | --- |
| Escape empty-handed | `ESCAPE_WITHOUT_FLASHBANG` | Escape without picking up a flashbang in that run. |
| Trust your ears | `ESCAPE_WITHOUT_DETECTOR` | Escape without picking up the detector in that run. |
| Buy some time | `FLASHBANG_HIT_MONSTER` | Hit the monster with a flashbang burst within 4 m and clear line of sight. This only affects the achievement and existing flash protection; it does not stun the monster. |
| In the dark | `ESCAPE_WITHOUT_BOILER` | Escape without restoring the boiler in that run. |
| Let there be light | `RESTORE_BOILER` | Complete boiler restoration. |
| So close | `CAUGHT_AFTER_EXIT_OPEN` | Open the unlocked exit, then get caught before escaping. |
| Unseen | `ESCAPE_UNDETECTED` | Escape without ever being detected in that run. |

## Local progress

Native builds save to `$XDG_DATA_HOME/the-specimen-bevy/achievements.json`, or `~/.local/share/the-specimen-bevy/achievements.json` when `XDG_DATA_HOME` is unset. Browser builds use `localStorage` key `the-specimen-bevy.achievements` for the site's origin. Private browsing or blocked storage can prevent persistence. Progress uses versioned JSON with a list of unlocked API names; unknown names are retained for forwards compatibility. A corrupt native file is backed up as `.corrupt` (or a numbered variant) before a new one is written. No file is changed when the backup fails.

Progress is local to that computer or browser origin unless a Steam-enabled build successfully syncs it. A Steam download does not automatically import a separate itch browser's localStorage. Clearing browser storage or deleting the native file removes the local copy.

## Optional Steam sync

The regular native and web builds do not need Steam. A native build made with `--features steam` tries to initialize Steamworks at startup; if Steam is unavailable, local achievements keep working. When Steam stats are ready, the adapter merges unlocked achievements in both directions without clearing either side. The local store remains a fallback, including for offline use. A Steam-connected account is not verified by this repository's tests.

Before using Steam sync, obtain the game's real Steam App ID through Steamworks, define all seven API names above in the Steamworks achievement configuration, and publish that configuration. Launch the Steam-enabled game through Steam after onboarding. For local development, a developer-owned `steam_appid.txt` next to the executable may be necessary. Do not ship a placeholder App ID or a development `steam_appid.txt` in itch releases. The release workflow currently builds without `--features steam`; distributing a Steam-enabled binary needs a separate build/release decision. Validate unlocks with a real Steam account and client before advertising Steam achievements.

Build check: `nix develop -c cargo check -p the-specimen-bevy --features steam --locked`.
