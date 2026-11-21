use std::sync::Mutex;

use mod_api::ClientHost;

use crate::enchant;
use crate::glow::brighter;
use crate::lights::{self, PRIORITY_DROPPED, PRIORITY_DROPPED_ENCHANT, PRIORITY_FRAME, PRIORITY_FRAME_ENCHANT, PRIORITY_HELD, PRIORITY_HELD_ENCHANT};
use crate::profiles::{self, LightProfile};
use crate::settings;

const HELD_OFFSET: f64 = 1.2;
const DROPPED_OFFSET: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
struct HeldLight {
    id: u32,
    profile: Option<LightProfile>,
    enchanted: bool,
    at: [f64; 3],
    is_player: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DroppedLight {
    profile: Option<LightProfile>,
    enchanted: bool,
    at: [f64; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FrameLight {
    profile: Option<LightProfile>,
    enchanted: bool,
    at: [f64; 3],
}

static MOBS: Mutex<Vec<HeldLight>> = Mutex::new(Vec::new());
static DROPPED: Mutex<Vec<DroppedLight>> = Mutex::new(Vec::new());
static FRAMES: Mutex<Vec<FrameLight>> = Mutex::new(Vec::new());

fn finite(at: [f64; 3]) -> bool {
    at.iter().all(|axis| axis.is_finite())
}

fn is_local(host: &dyn ClientHost, id: u32) -> bool {
    let Some(entity) = host.entity(id) else { return false; };
    host.local_player().is_some_and(|player| player.entity() == entity)
}

fn hand(stack: Option<(&str, bool)>) -> (Option<LightProfile>, bool) {
    match stack {
        Some((id, enchanted)) => (profiles::profile_for(id), enchanted),
        None => (None, false),
    }
}

pub(crate) fn set_mobs<'a>(
    mobs: impl IntoIterator<Item = (u32, [f64; 3], Option<(&'a str, bool)>, Option<(&'a str, bool)>, bool)>,
) {
    let Ok(mut cache) = MOBS.lock() else { return; };
    cache.clear();

    for (id, at, main, off, is_player) in mobs {
        if !finite(at) { continue; }
        let (main_profile, main_enchanted) = hand(main);
        let (off_profile, off_enchanted) = hand(off);
        let profile = brighter(main_profile, off_profile);
        let enchanted = main_enchanted || off_enchanted;
        if profile.is_none() && !enchanted { continue; }
        cache.push(HeldLight {
            id,
            profile,
            enchanted,
            at: [at[0], at[1] + HELD_OFFSET, at[2]],
            is_player,
        });
    }
}

pub(crate) fn set_dropped<'a>(items: impl IntoIterator<Item = ([f64; 3], &'a str, bool)>) {
    let Ok(mut cache) = DROPPED.lock() else { return; };
    cache.clear();

    for (at, id, enchanted) in items {
        if !finite(at) { continue; }
        let profile = profiles::profile_for(id);
        if profile.is_none() && !enchanted { continue; }
        cache.push(DroppedLight {
            profile,
            enchanted,
            at: [at[0], at[1] + DROPPED_OFFSET, at[2]],
        });
    }
}

/// The frame's item sits half a block out from the centre of the block it is attached to.
fn frame_position(pos: [i32; 3], step: [i32; 3]) -> [f64; 3] {
    [
        pos[0] as f64 + 0.5 + step[0] as f64 * 0.5,
        pos[1] as f64 + 0.5 + step[1] as f64 * 0.5,
        pos[2] as f64 + 0.5 + step[2] as f64 * 0.5,
    ]
}

pub(crate) fn set_frames<'a>(
    frames: impl IntoIterator<Item = ([i32; 3], Option<(&'a str, bool)>, [i32; 3])>,
) {
    let Ok(mut cache) = FRAMES.lock() else { return; };
    cache.clear();

    for (pos, item, step) in frames {
        let Some((id, enchanted)) = item else { continue; };
        let profile = profiles::profile_for(id);
        if profile.is_none() && !enchanted { continue; }
        let at = frame_position(pos, step);
        if !finite(at) { continue; }
        cache.push(FrameLight { profile, enchanted, at });
    }
}

pub(crate) fn emit(host: &dyn ClientHost, out: &mut Vec<lights::Candidate>, colors: bool) {
    let other_players = settings::other_players(host);
    let dropped_items = settings::dropped_items(host);
    let item_frames = settings::item_frames(host);
    let enchanted_glow = settings::enchanted_glow(host);

    if let Ok(mobs) = MOBS.lock() {
        for source in mobs.iter() {
            if is_local(host, source.id) { continue; }
            if source.is_player && !other_players { continue; }
            if let Some(profile) = source.profile {
                out.push(lights::profile(source.at, profile, colors, PRIORITY_HELD));
            }
            if source.enchanted && enchanted_glow {
                out.push(enchant::halo(source.at, colors, PRIORITY_HELD_ENCHANT));
            }
        }
    }

    if dropped_items
        && let Ok(dropped) = DROPPED.lock()
    {
        for source in dropped.iter() {
            if let Some(profile) = source.profile {
                out.push(lights::profile(source.at, profile, colors, PRIORITY_DROPPED));
            }
            if source.enchanted && enchanted_glow {
                out.push(enchant::halo(source.at, colors, PRIORITY_DROPPED_ENCHANT));
            }
        }
    }

    if item_frames
        && let Ok(frames) = FRAMES.lock()
    {
        for source in frames.iter() {
            if let Some(profile) = source.profile {
                out.push(lights::profile(source.at, profile, colors, PRIORITY_FRAME));
            }
            if source.enchanted && enchanted_glow {
                out.push(enchant::halo(source.at, colors, PRIORITY_FRAME_ENCHANT));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TORCH: LightProfile = LightProfile::new(14.0, 0xf5ac7b, 14.0, 0.70);

    #[test]
    fn dropped_snapshot_filters_invalid_and_replaces_previous_items() {
        set_dropped([([10.0, f64::NAN, -4.0], "minecraft:torch", false)]);
        assert!(DROPPED.lock().expect("dropped lock").is_empty());

        set_dropped([
            ([1.0, 64.0, -2.0], "minecraft:torch", false),
            ([4.0, 65.0, 8.0], "minecraft:stick", false),
            ([7.0, 66.0, 1.0], "minecraft:stick", true),
        ]);
        let dropped = DROPPED.lock().expect("dropped lock");
        assert_eq!(dropped.len(), 2);
        assert_eq!(dropped[0].at, [1.0, 64.25, -2.0]);
        assert_eq!(dropped[0].profile, Some(TORCH));
        assert!(!dropped[0].enchanted);
        assert_eq!(dropped[1].at, [7.0, 66.25, 1.0]);
        assert!(dropped[1].enchanted);
    }

    #[test]
    fn mobs_snapshot_keeps_luminous_and_enchanted_hands() {
        set_mobs([
            (1, [0.0, 64.0, 0.0], Some(("minecraft:torch", false)), None, true),
            (2, [4.0, 64.0, 0.0], Some(("minecraft:stick", true)), None, false),
            (3, [8.0, 64.0, 0.0], Some(("minecraft:stick", false)), None, false),
            (4, [f64::NAN, 64.0, 0.0], Some(("minecraft:torch", false)), None, false),
        ]);
        let mobs = MOBS.lock().expect("mobs lock");
        assert_eq!(mobs.len(), 2);
        assert_eq!(mobs[0].at, [0.0, 65.2, 0.0]);
        assert!(mobs[0].is_player);
        assert_eq!(mobs[0].profile, Some(TORCH));
        assert!(!mobs[1].is_player);
        assert!(mobs[1].enchanted);
        assert_eq!(mobs[1].profile, None);
    }

    #[test]
    fn frame_light_uses_the_face_offset() {
        assert_eq!(frame_position([10, 64, 20], [0, 0, -1]), [10.5, 64.5, 20.0]);

        set_frames([
            ([10, 64, 20], Some(("minecraft:torch", false)), [0, 0, -1]),
            ([11, 64, 20], Some(("minecraft:stick", true)), [1, 0, 0]),
            ([12, 64, 20], Some(("minecraft:stick", false)), [0, 1, 0]),
        ]);
        let frames = FRAMES.lock().expect("frames lock");
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].at, [10.5, 64.5, 20.0]);
        assert_eq!(frames[1].at, [12.0, 64.5, 20.5]);
    }
}
