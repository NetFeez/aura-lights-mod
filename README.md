# Aurora Lights

Native Horizon mod that adds dynamic glow to light sources in your hand, dropped luminous items and
items displayed in frames. It can also tint that light and add a faint violet halo to enchanted gear.

Repo: https://github.com/netfeez/aura-lights-mod
Companion shader repo: https://github.com/netfeez/aura-bloom-shader

## Installation

1. Build or download the `.hmod` from this repo.
2. Copy it into the `mods` folder of your Horizon instance and enable it.
3. As a native mod, the launcher will ask you to trust the code before running it.

The mod id is `netfeez_aurora_lights`.

## Settings

All settings appear in the game's mod menu and are in English when the game language is English.

| Setting | Default | What it does |
|---|---|---|
| Other players | On | Shows the held-item glow for nearby players. |
| Dropped items | On | Shows glow for replicated dropped luminous items, even without a server-side mod. |
| Light colors | Off | Tints this mod's glow and the colored block light from the companion shader. |
| Enchanted glow | On | Adds a violet halo to enchanted items and armor, including framed items. |
| Item frames | On | Shows glow for luminous items displayed in item frames. |

Turn on **Light colors** if you want colored light instead of white.

## Compatibility

This mod works with the Aurora Bloom shader repo. That shader reads the colored light emitted here
to light water, blocks and entities. Only one shader pack can be active at a time, so select that one
only.

Do not also enable **Horizon Extras > Dynamic Lights**: both systems would emit duplicate lights.

The enchantment halo cannot change the vanilla glint because the shader API does not expose that
mask. The mod emits the halo as colored light and the shader displays it.

## Languages

- English: `assets/netfeez_aurora_lights/lang/en_us.json`
- Spanish: `assets/netfeez_aurora_lights/lang/es_es.json`

Spanish game variants use the same `es_es.json` file.

## Verification

```sh
cargo horizon-mod check
cargo horizon-mod build
```

The resulting package is saved as `target/horizon-mod/netfeez_aurora_lights.hmod`.
