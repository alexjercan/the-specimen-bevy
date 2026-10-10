# Achievements

The game tracks eight visible achievements and one hidden achievement. Open **Achievements** below Credits on the main menu to see the locked/unlocked status, criteria, and icon for each one. The hidden achievement shows as `?` with a `?` icon and no criteria until it unlocks. New unlocks show a short in-game toast. Progress survives new runs and game restarts. The main-menu build saves progress; automated headless runs do not write player data.

| Achievement | Steam API name | Unlock condition |
| --- | --- | --- |
| Escaped | `ESCAPED` | Escape the facility. |
| Escape empty-handed | `ESCAPE_WITHOUT_FLASHBANG` | Escape without picking up a flashbang in that run. |
| Trust your ears | `ESCAPE_WITHOUT_DETECTOR` | Escape without picking up the detector in that run. |
| Buy some time | `FLASHBANG_HIT_MONSTER` | Hit the monster with a flashbang burst within 4 m and clear line of sight. This only affects the achievement and existing flash protection; it does not stun the monster. |
| In the dark | `ESCAPE_WITHOUT_BOILER` | Escape while the power is out, without restoring the boiler in that run. Escaping before the outage does not qualify. |
| Let there be light | `RESTORE_BOILER` | Complete boiler restoration. |
| So close | `CAUGHT_AFTER_EXIT_OPEN` | Open the unlocked exit, then get caught before escaping. |
| Unseen | `ESCAPE_UNDETECTED` | Escape without ever being detected in that run. |
| Out of your mind (hidden) | `ESCAPE_INSANE` | Escape in Insane mode. |

## Insane mode

Unlock all eight visible achievements to unlock Insane mode. The hidden achievement does not count toward this gate. Before the unlock, the main menu shows only **Play**, which starts Normal mode. After the unlock, a mode selector (`<  Normal  >`) appears on the same line as **Play**. It starts at Normal each time the main menu opens; use the left and right arrows to select Insane, then press **Play**. **Retry** after a game over or an escape restarts the same mode as the run that just ended.

| Setting | Normal | Insane |
| --- | --- | --- |
| Monster walk speed (`MonsterWalkSpeed`) | `Slow`, 0.5 m/s | `Fast`, 1 m/s |
| Monster chase speed (`MonsterChaseSpeed`) | `Normal`, sprint speed + 1 m/s (7 m/s) | `Fast`, twice sprint speed (12 m/s) |
| Monster walk hearing (`MonsterWalkHearing`) | `Short`, 3 m | `Long`, 6 m |
| Monster sprint hearing (`MonsterRunHearing`) | `Short`, 9 m | `Long`, 18 m |
| Sprint stamina | full, recharges | empty and permanently exhausted |
| Flashbang and detector pickups | present | absent |
| Facility power | outage after 30-90 s; boiler restores it | out from the start; boiler cannot be repaired |

An Insane escape also unlocks Escaped and any other visible escape achievements that the run satisfies.

The presets live in `crates/gameplay/src/presets/` (`normal.rs`, `insane.rs`) as `GamePreset` resources. Each monster tuning field is an enum whose `value()` gives the number in meters or meters per second. `sprint`, `flashbangs`, `detector`, and `blackout` are separate booleans.

## Local progress

Native builds save to `$XDG_DATA_HOME/the-specimen-bevy/achievements.json`, or `~/.local/share/the-specimen-bevy/achievements.json` when `XDG_DATA_HOME` is unset. Browser builds use `localStorage` key `the-specimen-bevy.achievements` for the site's origin. Private browsing or blocked storage can prevent persistence. Progress uses versioned JSON with a list of unlocked API names; unknown names are retained for forwards compatibility. A corrupt native file is backed up as `.corrupt` (or a numbered variant) before a new one is written. No file is changed when the backup fails.

Progress is local to that computer or browser origin unless a Steam-enabled build successfully syncs it. A Steam download does not automatically import a separate itch browser's localStorage. Clearing browser storage or deleting the native file removes the local copy.

## Optional Steam sync

The regular native and web builds do not need Steam. A native build made with `--features steam` tries to initialize Steamworks at startup; if Steam is unavailable, local achievements keep working. When Steam stats are ready, the adapter merges unlocked achievements in both directions without clearing either side. The local store remains a fallback, including for offline use. A Steam-connected account is not verified by this repository's tests.

Before using Steam sync, obtain the game's real Steam App ID through Steamworks, define all nine API names above in the Steamworks achievement configuration, mark `ESCAPE_INSANE` as hidden there, and publish that configuration. Launch the Steam-enabled game through Steam after onboarding. For local development, a developer-owned `steam_appid.txt` next to the executable may be necessary. Do not ship a placeholder App ID or a development `steam_appid.txt` in itch releases. The release workflow currently builds without `--features steam`; distributing a Steam-enabled binary needs a separate build/release decision. Validate unlocks with a real Steam account and client before advertising Steam achievements.

Build check: `nix develop -c cargo check -p the-specimen-bevy --features steam --locked`.
