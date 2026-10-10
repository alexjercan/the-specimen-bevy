# Halloween cosmetic update on a separate PR branch

- STATUS: OPEN
- PRIORITY: 0
- TAGS: backlog

## User facts

- Plan a Halloween update on a separate branch as a PR. Do not implement it yet; other wishlist work comes first.
- Consider alternate fuse pickup models (for example candy), lit pumpkins and other sparse orange decorations, and orange glowing monster eyes.
- Include a true/false way to turn the theme on or off. The user asked whether activation should be by date or by release; no activation policy has been approved yet.

## Agent findings

- A clean `halloween-update` sprout exists at `/home/alex/.cache/sprouts/the-specimen-bevy/halloween-update`, based on 4d23250. It does not include uncommitted master changes.
- Existing assets include `assets/facility/modules/fuse_pickup.glb` and `assets/monster/human_deer_animated.glb`; assess asset provenance and compatibility before making variants.
- A release-controlled cosmetic theme with an explicit switch, rather than date-only activation, was suggested but is not a user decision.

## Delivery

- After the user resumes this task, agree on art scope and activation policy; implement and review the seasonal cosmetics in the sprout, not directly on master.
- Keep fuse pickups recognizable, monster visibility fair, navigation clear, and the base game usable with the theme off.
- Add appropriate assets, credits/licenses, tests, and changelog notes. Commit and open a PR only when the user asks to proceed.

## Verification

- With the theme off, original assets and behavior remain unchanged. With it on, all selected decorations and variants load on native and browser builds without obscuring objectives or routes.
- Test asset loading and visibility in a real game run, as well as the relevant automated tests.

## Done when

- The approved cosmetic scope and switch are implemented and reviewed, required checks pass, and a PR from the Halloween branch is ready for review. No release or deployment is implied.
