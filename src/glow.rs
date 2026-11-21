use mod_api::{ClientHost, Hand, LivingRef};

use crate::lights::{self, PRIORITY_LOCAL_HELD};
use crate::profiles::{self, LightProfile};

const HELD_OFFSET: f64 = 1.2;

pub(crate) fn emit_held(host: &dyn ClientHost, out: &mut Vec<lights::Candidate>, colors: bool) {
    let Some(player) = host.local_player()         else { return; };
    let Some(living) = host.living(player.entity()) else { return; };
    let Some(at) = living.entity().position(host)   else { return; };
    let at = [at[0], at[1] + HELD_OFFSET, at[2]];

    let main = held_profile(host, &living, Hand::Main);
    let off = held_profile(host, &living, Hand::Off);
    let Some(profile) = brighter(main, off) else { return; };
    out.push(lights::profile(at, profile, colors, PRIORITY_LOCAL_HELD));
}

pub(crate) fn brighter(main: Option<LightProfile>, off: Option<LightProfile>) -> Option<LightProfile> {
    match (main, off) {
        (Some(main), Some(off)) => Some(if main.level >= off.level { main } else { off }),
        (Some(profile), None) | (None, Some(profile)) => Some(profile),
        (None, None) => None,
    }
}

fn held_profile(host: &dyn ClientHost, living: &LivingRef, hand: Hand) -> Option<LightProfile> {
    let stack = living.hand_item(host, hand)?;
    let id = stack.item().id(host);
    if id == "minecraft:air" { return None; }
    profiles::profile_for(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brighter_hand_wins() {
        let high = LightProfile::new(14.0, 0xffb85c, 14.0, 0.42);
        let low = LightProfile::new(10.0, 0x7cd6ff, 10.0, 0.3);
        assert_eq!(brighter(Some(high), Some(low)), Some(high));
        assert_eq!(brighter(None, Some(low)), Some(low));
        assert_eq!(brighter(None, None), None);
    }
}
