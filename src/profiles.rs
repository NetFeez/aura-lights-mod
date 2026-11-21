const WARM: u32 = 0xf5ac7b;
const AMBER: u32 = 0xdb9b51;
const COPPER: u32 = 0xecceab;
const COPPER_GREEN: u32 = 0x9cd477;
const OCHRE_FROGLIGHT: u32 = 0xfaf2b5;
const VERDANT_FROGLIGHT: u32 = 0xc4f4c8;
const PEARLESCENT_FROGLIGHT: u32 = 0xe18ce4;
const SEA: u32 = 0xbee1db;
const SOUL: u32 = 0x5ae0ea;
const PURPLE: u32 = 0xb358ef;
const AMETHYST: u32 = 0xc682ec;
const REDSTONE: u32 = 0xce5247;
const WHITE: u32 = 0xffffff;

pub(crate) const LIGHT_INTENSITY: f32 = 0.70;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LightProfile {
    pub(crate) level: f32,
    pub(crate) rgb: u32,
    pub(crate) radius: f32,
    pub(crate) intensity: f32,
}

impl LightProfile {
    pub(crate) const fn new(level: f32, rgb: u32, radius: f32, intensity: f32) -> Self {
        Self { level, rgb, radius, intensity }
    }

    const fn from_level(rgb: u32, level: u8) -> Self {
        let level = level as f32;
        Self::new(level, rgb, level, LIGHT_INTENSITY)
    }
}

pub(crate) fn profile_for(item_id: &str) -> Option<LightProfile> {
    match item_id {
        "minecraft:torch" | "minecraft:wall_torch" => Some(LightProfile::from_level(WARM, 14)),
        "minecraft:glow_berries" | "minecraft:cave_vines" | "minecraft:cave_vines_plant" => Some(LightProfile::from_level(AMBER, 14)),
        "minecraft:soul_torch" | "minecraft:soul_wall_torch" => Some(LightProfile::from_level(SOUL, 10)),
        "minecraft:soul_lantern" => Some(LightProfile::from_level(SOUL, 10)),
        "minecraft:crying_obsidian" => Some(LightProfile::from_level(PURPLE, 10)),
        "minecraft:soul_campfire" => Some(LightProfile::from_level(SOUL, 10)),
        "minecraft:lantern" => Some(LightProfile::from_level(WARM, 15)),
        "minecraft:glowstone" => Some(LightProfile::from_level(WHITE, 15)),
        "minecraft:sea_lantern" => Some(LightProfile::from_level(SEA, 15)),
        "minecraft:shroomlight" => Some(LightProfile::from_level(AMBER, 15)),
        "minecraft:pearlescent_froglight" => Some(LightProfile::from_level(PEARLESCENT_FROGLIGHT, 15)),
        "minecraft:verdant_froglight" => Some(LightProfile::from_level(VERDANT_FROGLIGHT, 15)),
        "minecraft:ochre_froglight" => Some(LightProfile::from_level(OCHRE_FROGLIGHT, 15)),
        "minecraft:end_rod" => Some(LightProfile::from_level(PEARLESCENT_FROGLIGHT, 14)),
        "minecraft:beacon" => Some(LightProfile::from_level(WHITE, 15)),
        "minecraft:campfire" => Some(LightProfile::from_level(WARM, 15)),
        "minecraft:jack_o_lantern" => Some(LightProfile::from_level(AMBER, 15)),
        "minecraft:conduit" => Some(LightProfile::from_level(WHITE, 15)),
        "minecraft:lava_bucket" | "minecraft:lava" => Some(LightProfile::from_level(AMBER, 15)),
        "minecraft:fire" => Some(LightProfile::from_level(WARM, 15)),
        "minecraft:soul_fire" => Some(LightProfile::from_level(SOUL, 10)),
        "minecraft:redstone_lamp" => Some(LightProfile::from_level(COPPER, 15)),
        "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => Some(LightProfile::from_level(REDSTONE, 7)),
        "minecraft:glow_lichen" => Some(LightProfile::from_level(VERDANT_FROGLIGHT, 7)),
        "minecraft:enchanting_table" => Some(LightProfile::from_level(PURPLE, 7)),
        "minecraft:ender_chest" => Some(LightProfile::from_level(WHITE, 7)),
        "minecraft:sea_pickle" => Some(LightProfile::from_level(SEA, 6)),
        "minecraft:sculk_catalyst" => Some(LightProfile::from_level(SOUL, 6)),
        "minecraft:magma_block" => Some(LightProfile::from_level(AMBER, 3)),
        "minecraft:brewing_stand" => Some(LightProfile::from_level(WHITE, 1)),
        "minecraft:brown_mushroom" => Some(LightProfile::from_level(WHITE, 1)),
        "minecraft:dragon_egg" => Some(LightProfile::from_level(WHITE, 1)),
        "minecraft:sculk_sensor" => Some(LightProfile::from_level(SOUL, 1)),
        "minecraft:calibrated_sculk_sensor" => Some(LightProfile::from_level(WHITE, 1)),
        "minecraft:light" => Some(LightProfile::from_level(WHITE, 15)),
        "minecraft:copper_lantern" => Some(LightProfile::from_level(COPPER_GREEN, 15)),
        "minecraft:copper_torch" => Some(LightProfile::from_level(COPPER_GREEN, 14)),
        "minecraft:exposed_copper_lantern"
        | "minecraft:weathered_copper_lantern"
        | "minecraft:oxidized_copper_lantern"
        | "minecraft:waxed_copper_lantern"
        | "minecraft:waxed_exposed_copper_lantern"
        | "minecraft:waxed_weathered_copper_lantern"
        | "minecraft:waxed_oxidized_copper_lantern" => Some(LightProfile::from_level(COPPER, 15)),
        "minecraft:amethyst_cluster" => Some(LightProfile::from_level(AMETHYST, 5)),
        "minecraft:large_amethyst_bud" => Some(LightProfile::from_level(AMETHYST, 4)),
        "minecraft:medium_amethyst_bud" => Some(LightProfile::from_level(AMETHYST, 2)),
        "minecraft:small_amethyst_bud" => Some(LightProfile::from_level(AMETHYST, 1)),
        "minecraft:firefly_bush" => Some(LightProfile::from_level(COPPER, 2)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL_LEVELS: [(&str, f32); 36] = [
        ("minecraft:torch", 14.0),
        ("minecraft:wall_torch", 14.0),
        ("minecraft:glow_berries", 14.0),
        ("minecraft:soul_torch", 10.0),
        ("minecraft:soul_lantern", 10.0),
        ("minecraft:crying_obsidian", 10.0),
        ("minecraft:soul_campfire", 10.0),
        ("minecraft:lantern", 15.0),
        ("minecraft:glowstone", 15.0),
        ("minecraft:sea_lantern", 15.0),
        ("minecraft:shroomlight", 15.0),
        ("minecraft:pearlescent_froglight", 15.0),
        ("minecraft:verdant_froglight", 15.0),
        ("minecraft:ochre_froglight", 15.0),
        ("minecraft:end_rod", 14.0),
        ("minecraft:beacon", 15.0),
        ("minecraft:campfire", 15.0),
        ("minecraft:jack_o_lantern", 15.0),
        ("minecraft:conduit", 15.0),
        ("minecraft:lava_bucket", 15.0),
        ("minecraft:redstone_torch", 7.0),
        ("minecraft:glow_lichen", 7.0),
        ("minecraft:enchanting_table", 7.0),
        ("minecraft:ender_chest", 7.0),
        ("minecraft:sea_pickle", 6.0),
        ("minecraft:sculk_catalyst", 6.0),
        ("minecraft:magma_block", 3.0),
        ("minecraft:brewing_stand", 1.0),
        ("minecraft:brown_mushroom", 1.0),
        ("minecraft:light", 15.0),
        ("minecraft:copper_lantern", 15.0),
        ("minecraft:copper_torch", 14.0),
        ("minecraft:amethyst_cluster", 5.0),
        ("minecraft:small_amethyst_bud", 1.0),
        ("minecraft:calibrated_sculk_sensor", 1.0),
        ("minecraft:firefly_bush", 2.0),
    ];

    #[test]
    fn supported_items_have_bounded_profiles() {
        for (id, level) in REAL_LEVELS {
            let profile = profile_for(id).unwrap_or_else(|| panic!("missing profile for {id}"));
            assert_eq!(profile.level, level, "{id}");
            assert_eq!(profile.radius, level, "{id}");
            assert_eq!(profile.intensity, LIGHT_INTENSITY, "{id}");
            assert!(profile.rgb <= 0xffffff);
            assert!((0.0..=16.0).contains(&profile.radius), "{id}");
        }
    }

    #[test]
    fn intensity_matches_shader_after_gamma_decode() {
        assert!((LIGHT_INTENSITY.powf(2.2) - 0.45).abs() < 0.01);
    }

    #[test]
    fn canonical_colors_are_used() {
        for (id, rgb) in [
            ("minecraft:torch", WARM),
            ("minecraft:lantern", WARM),
            ("minecraft:fire", WARM),
            ("minecraft:campfire", WARM),
            ("minecraft:shroomlight", AMBER),
            ("minecraft:redstone_lamp", COPPER),
            ("minecraft:copper_lantern", COPPER_GREEN),
            ("minecraft:soul_torch", SOUL),
            ("minecraft:sculk_sensor", SOUL),
            ("minecraft:sea_lantern", SEA),
            ("minecraft:enchanting_table", PURPLE),
            ("minecraft:amethyst_cluster", AMETHYST),
            ("minecraft:redstone_torch", REDSTONE),
            ("minecraft:glowstone", WHITE),
        ] {
            assert_eq!(profile_for(id).expect(id).rgb, rgb, "{id}");
        }
    }

    #[test]
    fn froglights_have_distinct_colors() {
        let ochre = profile_for("minecraft:ochre_froglight").expect("ochre");
        let verdant = profile_for("minecraft:verdant_froglight").expect("verdant");
        let pearlescent = profile_for("minecraft:pearlescent_froglight").expect("pearlescent");
        assert_eq!(ochre.rgb, OCHRE_FROGLIGHT);
        assert_eq!(verdant.rgb, VERDANT_FROGLIGHT);
        assert_eq!(pearlescent.rgb, PEARLESCENT_FROGLIGHT);
        assert_ne!(ochre.rgb, verdant.rgb);
        assert_ne!(ochre.rgb, pearlescent.rgb);
        assert_ne!(verdant.rgb, pearlescent.rgb);
    }

    #[test]
    fn placed_light_blocks_have_profiles() {
        for id in [
            "minecraft:cave_vines",
            "minecraft:lava",
            "minecraft:fire",
            "minecraft:soul_fire",
            "minecraft:redstone_lamp",
        ] {
            assert!(profile_for(id).is_some(), "{id}");
        }
    }

    #[test]
    fn brighter_sources_reach_further() {
        let torch = profile_for("minecraft:torch").expect("torch profile");
        let mushroom = profile_for("minecraft:brown_mushroom").expect("mushroom profile");
        assert!(torch.radius > mushroom.radius);
        assert_eq!(torch.intensity, mushroom.intensity);
    }

    #[test]
    fn unrelated_items_have_no_profile() {
        assert_eq!(profile_for("minecraft:stick"), None);
        assert_eq!(profile_for(""), None);
    }
}
