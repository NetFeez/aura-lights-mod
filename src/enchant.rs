use std::sync::OnceLock;

use mod_api::{ClientHost, ComponentId, EquipmentSlot, Hand, ItemResource, Tag};

use crate::lights::{self, PRIORITY_LOCAL_ENCHANT};
use crate::settings;

const COLOR: u32 = 0xb358ef;
pub(crate) const RADIUS: f32 = 3.5;
pub(crate) const INTENSITY: f32 = 0.22;
const OFFSET: f64 = 1.1;

static ENCHANTMENTS: OnceLock<ComponentId> = OnceLock::new();

fn enchantments() -> &'static ComponentId {
    ENCHANTMENTS.get_or_init(|| ComponentId::new("minecraft:enchantments").expect("enchantments id"))
}

fn has_levels(value: Option<Tag>) -> bool {
    matches!(value, Some(Tag::Compound(levels)) if !levels.is_empty())
}

fn enchanted(host: &dyn ClientHost, stack: &mod_api::ItemStack) -> bool {
    has_levels(ItemResource::of(stack).value(host, enchantments()))
}

/// A halo candidate shared by held, dropped and framed enchanted stacks.
pub(crate) fn halo(at: [f64; 3], colors: bool, priority: u8) -> lights::Candidate {
    let rgb = if colors { COLOR } else { lights::WHITE };
    lights::Candidate::new(at, rgb, RADIUS, INTENSITY, priority)
}

pub(crate) fn emit(host: &dyn ClientHost, out: &mut Vec<lights::Candidate>, colors: bool) {
    if !settings::enchanted_glow(host) { return; }
    let Some(player) = host.local_player()         else { return; };
    let Some(living) = host.living(player.entity()) else { return; };

    let hands = [Hand::Main, Hand::Off].into_iter()
        .filter_map(|hand| living.hand_item(host, hand));
    let armor = [EquipmentSlot::Feet, EquipmentSlot::Legs, EquipmentSlot::Chest, EquipmentSlot::Head].into_iter()
        .filter_map(|slot| living.equipment(host, slot));
    if !hands.chain(armor).any(|stack| enchanted(host, &stack)) { return; }

    let Some(at) = living.entity().position(host) else { return; };
    out.push(halo([at[0], at[1] + OFFSET, at[2]], colors, PRIORITY_LOCAL_ENCHANT));
}

#[cfg(test)]
mod tests {
    use super::*;
    use mod_api::Compound;

    #[test]
    fn only_nonempty_enchantments_glow() {
        assert!(!has_levels(None));
        assert!(!has_levels(Some(Tag::String("minecraft:sharpness".into()))));
        assert!(!has_levels(Some(Tag::Compound(Default::default()))));

        let mut levels = Compound::new();
        levels.insert("minecraft:sharpness".into(), Tag::Int(1));
        assert!(has_levels(Some(Tag::Compound(levels))));
    }

    #[test]
    fn halo_respects_colors_setting() {
        assert_eq!(halo([0.0; 3], true, PRIORITY_LOCAL_ENCHANT).rgb, COLOR);
        assert_eq!(halo([0.0; 3], false, PRIORITY_LOCAL_ENCHANT).rgb, lights::WHITE);
    }
}
