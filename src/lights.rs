use mod_api::{ColoredLight, VisualsEvent};

use crate::profiles::LightProfile;

pub(crate) const MAX_MOD_LIGHTS: usize = 8;

// Lower values win when more sources than the engine's limit are visible.
pub(crate) const PRIORITY_LOCAL_HELD: u8 = 0;
pub(crate) const PRIORITY_LOCAL_ENCHANT: u8 = 1;
pub(crate) const PRIORITY_HELD: u8 = 2;
pub(crate) const PRIORITY_HELD_ENCHANT: u8 = 3;
pub(crate) const PRIORITY_DROPPED: u8 = 4;
pub(crate) const PRIORITY_DROPPED_ENCHANT: u8 = 5;
pub(crate) const PRIORITY_FRAME: u8 = 6;
pub(crate) const PRIORITY_FRAME_ENCHANT: u8 = 7;

pub(crate) const WHITE: u32 = 0xffffff;

pub(crate) struct Candidate {
    at: [f64; 3],
    pub(crate) rgb: u32,
    radius: f32,
    intensity: f32,
    priority: u8,
}

impl Candidate {
    pub(crate) fn new(at: [f64; 3], rgb: u32, radius: f32, intensity: f32, priority: u8) -> Self {
        Self { at, rgb, radius, intensity, priority }
    }
}

pub(crate) fn profile(at: [f64; 3], profile: LightProfile, colors: bool, priority: u8) -> Candidate {
    let rgb = if colors { profile.rgb } else { WHITE };
    Candidate::new(at, rgb, profile.radius, profile.intensity, priority)
}

pub(crate) fn flush(event: &mut VisualsEvent<'_>, mut candidates: Vec<Candidate>) {
    candidates.sort_by_key(|candidate| candidate.priority);
    for candidate in candidates.into_iter().take(MAX_MOD_LIGHTS) {
        event.lights.push(ColoredLight::new(
            candidate.at, candidate.rgb, candidate.radius, candidate.intensity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_honors_colors_setting() {
        let torch = LightProfile::new(14.0, 0xffb85c, 12.4, 0.95);
        let colored = profile([0.0; 3], torch, true, PRIORITY_HELD);
        let white = profile([0.0; 3], torch, false, PRIORITY_HELD);
        assert_eq!(colored.rgb, 0xffb85c);
        assert_eq!(white.rgb, WHITE);
    }

    #[test]
    fn flush_keeps_highest_priority_and_respects_engine_limit() {
        let mut lights = Vec::new();
        let mut rims = Vec::new();
        let candidates = (0..12)
            .map(|index| Candidate::new([index as f64; 3], index as u32, 1.0, 0.5, index))
            .collect();
        flush(&mut VisualsEvent { rims: &mut rims, lights: &mut lights }, candidates);

        assert_eq!(lights.len(), MAX_MOD_LIGHTS);
        assert_eq!(lights[0].rgb, 0);
        assert_eq!(lights[1].rgb, 1);
        assert_eq!(lights[7].rgb, 7);
    }
}
