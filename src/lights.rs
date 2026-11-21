use std::cell::Cell;

use mod_api::{ColoredLight, VisualsEvent};

use crate::light_field::{balance, FieldLight};
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

/// The engine keeps only lights within 32 blocks of the player; matching that here stops an
/// out-of-range source from occupying one of the eight slots before the engine discards it.
const PLAYER_RANGE_SQ: f64 = 32.0 * 32.0;

type Rank = (bool, f64, f64, f64, u8);

#[derive(Clone, Copy)]
struct Pose {
    position: [f64; 3],
    yaw: f32,
    pitch: f32,
}

// camera_pose asks for this frame's pose, then visuals ranks against it. Both run on the render
// thread; a thread-local keeps the frame state out of global state and test threads independent.
thread_local! {
    static POSE: Cell<Option<Pose>> = const { Cell::new(None) };
}

pub(crate) fn set_pose(position: [f64; 3], yaw: f32, pitch: f32) {
    POSE.with(|pose| pose.set(Some(Pose { position, yaw, pitch })));
}

#[cfg(test)]
fn clear_pose() {
    POSE.with(|pose| pose.set(None));
}

fn current_pose() -> Option<Pose> {
    POSE.with(Cell::get)
}

#[derive(Clone)]
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

/// View direction for yaw/pitch in degrees: +Z at yaw 0, -X at yaw 90, up when pitch is negative.
fn forward(yaw: f32, pitch: f32) -> [f64; 3] {
    let yaw = (yaw as f64).to_radians();
    let pitch = (pitch as f64).to_radians();
    let flat = pitch.cos();
    [-yaw.sin() * flat, -pitch.sin(), yaw.cos() * flat]
}

fn in_player_range(candidate: &Candidate, player_at: Option<[f64; 3]>) -> bool {
    if !candidate.at.iter().all(|value| value.is_finite()) {
        return false;
    }
    let Some(player_at) = player_at else { return true; };
    let squared = candidate.at
        .iter()
        .zip(player_at)
        .map(|(at, player)| (at - player).powi(2))
        .sum::<f64>();
    squared <= PLAYER_RANGE_SQ
}

/// Rank by how much of a source's light can reach this view. The first key keeps the local
/// player's own light first; after that the nearest edge of the light's reach and its angle to
/// the view direction decide. A source behind the camera is ranked last, never dropped: the
/// engine's player range is the only hard limit, so a turn cannot hide a light that is in view.
fn rank(candidate: &Candidate, eye: [f64; 3], forward: [f64; 3]) -> Rank {
    let delta = [
        candidate.at[0] - eye[0],
        candidate.at[1] - eye[1],
        candidate.at[2] - eye[2],
    ];
    let distance = delta.iter().map(|value| value * value).sum::<f64>().sqrt();
    let along = delta[0] * forward[0] + delta[1] * forward[1] + delta[2] * forward[2];
    let facing = if distance > 0.0 { (along / distance).clamp(-1.0, 1.0) } else { 1.0 };
    let angle = (1.0 - facing) * 0.5;
    let reach = (distance - f64::from(candidate.radius)).max(0.0);
    let strength = -f64::from(candidate.radius) * f64::from(candidate.intensity);
    (
        candidate.priority != PRIORITY_LOCAL_HELD,
        strength,
        reach,
        angle,
        candidate.priority,
    )
}

fn by_rank((a, _): &(Rank, Candidate), (b, _): &(Rank, Candidate)) -> std::cmp::Ordering {
    a.0.cmp(&b.0)
        .then_with(|| a.1.total_cmp(&b.1))
        .then_with(|| a.2.total_cmp(&b.2))
        .then_with(|| a.3.total_cmp(&b.3))
        .then_with(|| a.4.cmp(&b.4))
}

/// Emit at most `MAX_MOD_LIGHTS` sources. The engine's player range is applied first, then the
/// remaining sources are ranked against this frame's camera pose before the cap. Overlapping
/// sources are scaled so the engine's sum matches the brightest light instead of adding up.
/// Before the first `camera_pose` event the previous source-kind order applies.
pub(crate) fn flush(event: &mut VisualsEvent<'_>, candidates: Vec<Candidate>, player_at: Option<[f64; 3]>) {
    let view = current_pose().map(|pose| (pose.position, forward(pose.yaw, pose.pitch)));
    let mut ranked: Vec<_> = candidates
        .into_iter()
        .filter(|candidate| in_player_range(candidate, player_at))
        .filter_map(|candidate| match view {
            Some((eye, forward)) => Some((rank(&candidate, eye, forward), candidate)),
            None => Some((
                (
                    true,
                    -f64::from(candidate.radius) * f64::from(candidate.intensity),
                    0.0,
                    0.0,
                    candidate.priority,
                ),
                candidate,
            )),
        })
        .collect();
    ranked.sort_by(by_rank);
    ranked.truncate(MAX_MOD_LIGHTS);

    let mut lights: Vec<FieldLight> = ranked
        .into_iter()
        .map(|(_, candidate)| FieldLight::new(
            candidate.at,
            candidate.rgb,
            candidate.radius,
            candidate.intensity,
            candidate.priority == PRIORITY_LOCAL_HELD,
        ))
        .collect();
    balance(&mut lights);
    for light in lights {
        event.lights.push(ColoredLight::new(
            light.at, light.rgb, light.radius, light.intensity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TORCH: LightProfile = LightProfile::new(14.0, 0xffb85c, 14.0, 0.95);

    fn flush_with(player_at: Option<[f64; 3]>, candidates: Vec<Candidate>) -> Vec<ColoredLight> {
        let mut lights = Vec::new();
        let mut rims = Vec::new();
        flush(&mut VisualsEvent { rims: &mut rims, lights: &mut lights }, candidates, player_at);
        lights
    }

    #[test]
    fn profile_honors_colors_setting() {
        let colored = profile([0.0; 3], TORCH, true, PRIORITY_HELD);
        let white = profile([0.0; 3], TORCH, false, PRIORITY_HELD);
        assert_eq!(colored.rgb, 0xffb85c);
        assert_eq!(white.rgb, WHITE);
    }

    #[test]
    fn flush_keeps_highest_priority_and_respects_engine_limit() {
        clear_pose();
        let candidates = (0..12)
            .map(|index| Candidate::new([index as f64 * 3.0; 3], index as u32, 1.0, 0.5, index))
            .collect();
        let lights = flush_with(None, candidates);

        assert_eq!(lights.len(), MAX_MOD_LIGHTS);
        assert_eq!(lights[0].rgb, 0);
        assert_eq!(lights[1].rgb, 1);
        assert_eq!(lights[7].rgb, 7);
    }

    #[test]
    fn flush_drops_sources_beyond_the_player_range() {
        clear_pose();
        let candidates = vec![
            profile([0.0, 64.0, 40.0], TORCH, false, PRIORITY_DROPPED),
            profile([0.0, 64.0, -4.0], TORCH, false, PRIORITY_FRAME),
        ];
        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);

        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].at, [0.0, 64.0, -4.0]);
    }

    #[test]
    fn flush_balances_near_lights_instead_of_turning_one_off() {
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let bright = LightProfile::new(15.0, 0xffffff, 15.0, 0.95);
        let dim = LightProfile::new(10.0, 0xff0000, 10.0, 0.95);
        let candidates = vec![
            profile([0.0, 64.0, 1.0], dim, false, PRIORITY_DROPPED),
            profile([0.0, 64.0, 2.0], bright, false, PRIORITY_DROPPED),
        ];

        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);
        assert_eq!(lights.len(), 2);
        assert!(lights.iter().all(|light| light.intensity > 0.0));
        assert!(lights.iter().all(|light| light.intensity < 0.95));
        assert_eq!(lights[0].radius, 15.0);
        assert_eq!(lights[1].radius, 10.0);
    }

    #[test]
    fn flush_keeps_separate_lights_that_do_not_overlap_locally() {
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let candidates = vec![
            profile([0.0, 64.0, 1.0], TORCH, false, PRIORITY_DROPPED),
            profile([0.0, 64.0, 5.0], TORCH, false, PRIORITY_FRAME),
        ];

        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);
        assert_eq!(lights.len(), 2);
    }

    #[test]
    fn flush_keeps_the_local_hand_full_beside_a_nearby_frame() {
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let candidates = vec![
            profile([0.0, 64.0, 1.0], TORCH, false, PRIORITY_LOCAL_HELD),
            profile([0.0, 64.0, 2.0], TORCH, false, PRIORITY_FRAME),
        ];

        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);
        assert_eq!(lights.len(), 2);
        assert_eq!(lights[0].at, [0.0, 64.0, 1.0]);
        assert!(lights[0].intensity > 0.94);
        assert!(lights[1].intensity < 0.95);
    }

    #[test]
    fn flush_ranks_against_the_current_render_pose() {
        let candidates = vec![
            profile([0.0, 64.0, 10.0], TORCH, false, PRIORITY_FRAME),
            profile([-10.0, 64.0, 0.0], TORCH, false, PRIORITY_DROPPED),
        ];
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let facing_z = flush_with(Some([0.0, 64.0, 0.0]), candidates.clone());
        set_pose([0.0, 64.0, 0.0], 90.0, 0.0);
        let facing_negative_x = flush_with(Some([0.0, 64.0, 0.0]), candidates);

        assert_eq!(facing_z[0].at, [0.0, 64.0, 10.0]);
        assert_eq!(facing_negative_x[0].at, [-10.0, 64.0, 0.0]);
    }

    #[test]
    fn flush_keeps_the_local_hand_when_frames_fill_the_cap() {
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let mut candidates = vec![profile(
            [0.0, 64.0, 1.0],
            TORCH,
            false,
            PRIORITY_LOCAL_HELD,
        )];
        candidates.extend((0..9).map(|index| {
            profile(
                [(index as f64 - 4.0) * 3.0, 64.0, 8.0 + index as f64 * 3.0],
                TORCH,
                false,
                PRIORITY_FRAME,
            )
        }));

        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);
        assert_eq!(lights.len(), MAX_MOD_LIGHTS);
        assert_eq!(lights[0].at, [0.0, 64.0, 1.0]);
    }

    #[test]
    fn flush_does_not_let_a_weak_halo_push_a_real_light_out_at_the_cap() {
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let mut candidates: Vec<_> = (0..8)
            .map(|index| profile(
                [(index as f64 - 4.0) * 3.0, 64.0, 20.0],
                TORCH,
                false,
                PRIORITY_FRAME,
            ))
            .collect();
        candidates.push(crate::enchant::halo(
            [0.0, 64.0, 1.0], false, PRIORITY_LOCAL_ENCHANT,
        ));

        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);
        assert_eq!(lights.len(), MAX_MOD_LIGHTS);
        assert!(lights.iter().all(|light| light.radius == 14.0));
    }

    #[test]
    fn flush_prefers_a_nearby_source_over_distant_ones() {
        set_pose([0.0, 64.0, 0.0], 0.0, 0.0);
        let mut candidates: Vec<_> = (0..8)
            .map(|index| profile([(index as f64 - 4.0) * 3.0, 64.0, 30.0], TORCH, false, PRIORITY_HELD))
            .collect();
        candidates.push(profile([0.0, 64.0, 2.0], TORCH, false, PRIORITY_DROPPED));

        let lights = flush_with(Some([0.0, 64.0, 0.0]), candidates);
        assert_eq!(lights.len(), MAX_MOD_LIGHTS);
        assert!(lights.iter().any(|light| light.at == [0.0, 64.0, 2.0]));
    }
}
