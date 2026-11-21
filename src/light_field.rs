//! Turns the engine's additive coloured lights into a "brightest light wins" field.
//!
//! The WGSL does `rgb += color.rgb * max(0, 1 - dist / radius) * intensity` for every light and
//! clamps the sum to 1. Minecraft's block light instead takes the brightest source. This module
//! fits one scale per light so the engine's sum stays close to the per-channel maximum without
//! touching the engine or a shader. It runs on the CPU over the already capped list.

const ITERATIONS: usize = 16;
const DAMPING: f64 = 0.6;
const EPSILON: f64 = 1e-9;

/// A light below this share of the strongest overlapping light is visually irrelevant there.
/// It is pushed towards zero instead of being allowed to brighten and dim a much brighter one.
const WEAK_BELOW: f64 = 0.25;
/// A light above this share is a real contributor and keeps its full strength unless the fit
/// needs otherwise. Between the two bounds the preference changes smoothly, so walking past a
/// light never flips a switch.
const STRONG_ABOVE: f64 = 0.60;
/// How much the "strong lights win" preference competes with the raw fit.
const REGULARIZATION: f64 = 0.12;

/// The local hand keeps its full strength only while it can actually compete with the lights
/// around it. A weaker held soul torch must not flatten a nearby glowstone frame.
const HAND_WEAK_BELOW: f64 = 0.70;
const HAND_STRONG_ABOVE: f64 = 0.95;
const HAND_BOOST: f64 = 120.0;

#[derive(Clone)]
pub(crate) struct FieldLight {
    pub(crate) at: [f64; 3],
    pub(crate) rgb: u32,
    pub(crate) radius: f32,
    pub(crate) intensity: f32,
    /// 1 for the local hand, 0 for every other source. It strengthens the hand's preference but
    /// never makes it unconditional: a clearly brighter frame still wins.
    pub(crate) hand: f64,
}

impl FieldLight {
    pub(crate) fn new(
        at: [f64; 3],
        rgb: u32,
        radius: f32,
        intensity: f32,
        hand: bool,
    ) -> Self {
        Self { at, rgb, radius, intensity, hand: if hand { 1.0 } else { 0.0 } }
    }

    fn color(&self) -> [f64; 3] {
        [
            f64::from((self.rgb >> 16) & 0xff) / 255.0,
            f64::from((self.rgb >> 8) & 0xff) / 255.0,
            f64::from(self.rgb & 0xff) / 255.0,
        ]
    }

    /// Peak channel value at the light's centre. Used only to compare strengths, not to paint.
    fn magnitude(&self) -> f64 {
        let color = self.color();
        f64::from(self.intensity) * color[0].max(color[1]).max(color[2])
    }

    /// Peak strength times reach. Comparing only peak brightness would let a short, weak soul
    /// torch outrank a long-range glowstone frame just because their colours are similar.
    fn influence(&self) -> f64 {
        self.magnitude() * f64::from(self.radius)
    }

    /// The same falloff the engine's WGSL uses: `rgb * intensity * max(0, 1 - d / radius)`.
    fn contribution(&self, at: [f64; 3]) -> [f64; 3] {
        if self.radius <= 0.0 || self.intensity <= 0.0 {
            return [0.0; 3];
        }
        let distance = self.at
            .iter()
            .zip(at)
            .map(|(light, at)| (light - at).powi(2))
            .sum::<f64>()
            .sqrt();
        let falloff = (1.0 - distance / f64::from(self.radius)).max(0.0);
        let strength = falloff * f64::from(self.intensity);
        let color = self.color();
        [color[0] * strength, color[1] * strength, color[2] * strength]
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// How much two lights overlap, from coincident (1) to touching at the edge (0). Continuous, so a
/// moving light fades its influence in and out instead of popping between two states.
fn overlap(a: &FieldLight, b: &FieldLight) -> f64 {
    let reach = f64::from(a.radius) + f64::from(b.radius);
    if reach <= 0.0 {
        return 0.0;
    }
    (1.0 - distance(a.at, b.at) / reach).clamp(0.0, 1.0)
}

fn smoothstep(low: f64, high: f64, x: f64) -> f64 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn strongest_share(light: &FieldLight, lights: &[FieldLight]) -> f64 {
    let own = light.influence();
    let strongest = lights.iter()
        .map(|other| other.influence() * overlap(light, other))
        .fold(0.0f64, f64::max);
    (own / strongest.max(EPSILON)).min(1.0)
}

/// Preferred scale per light: full for lights that can compete with the strongest light they
/// touch, zero for a much weaker one that would otherwise brighten and dim its neighbours.
fn preferred_scales(lights: &[FieldLight]) -> Vec<f64> {
    lights.iter().map(|light| {
        let share = strongest_share(light, lights);
        if light.hand > 0.0 {
            smoothstep(HAND_WEAK_BELOW, HAND_STRONG_ABOVE, share)
        } else {
            smoothstep(WEAK_BELOW, STRONG_ABOVE, share)
        }
    }).collect()
}

/// Each light's centre plus several points on every segment. Midpoints alone are not enough when
/// radii differ: a short-range halo and a long-range torch meet close to the halo, not halfway.
fn sample_points(lights: &[FieldLight]) -> Vec<[f64; 3]> {
    let mut points = Vec::with_capacity(lights.len() * (1 + 3 * lights.len()));
    points.extend(lights.iter().map(|light| light.at));
    for (index, light) in lights.iter().enumerate() {
        for other in lights.iter().skip(index + 1) {
            for fraction in [0.25, 0.5, 0.75] {
                points.push([
                    light.at[0] + (other.at[0] - light.at[0]) * fraction,
                    light.at[1] + (other.at[1] - light.at[1]) * fraction,
                    light.at[2] + (other.at[2] - light.at[2]) * fraction,
                ]);
            }
        }
    }
    points
}

/// Fit one scale per light. The data term pulls the engine's sum towards the brightest original
/// light at each sample; the preference term keeps comparable lights alive and fades a much
/// weaker one out. The preference is a smooth function of position and strength, so the result
/// changes smoothly as the player walks and cannot produce hard on/off flicker.
pub(crate) fn balance(lights: &mut [FieldLight]) {
    if lights.len() < 2 {
        return;
    }

    let samples = sample_points(lights);
    let mut contributions = Vec::with_capacity(samples.len());
    let mut targets = Vec::with_capacity(samples.len());
    for at in &samples {
        let values: Vec<[f64; 3]> = lights.iter().map(|light| light.contribution(*at)).collect();
        let mut target = [0.0f64; 3];
        for value in &values {
            for channel in 0..3 {
                target[channel] = target[channel].max(value[channel]);
            }
        }
        contributions.push(values);
        targets.push(target);
    }

    let data_strength: Vec<f64> = (0..lights.len()).map(|index| {
        samples.iter().enumerate()
            .map(|(sample, _)| dot(contributions[sample][index], contributions[sample][index]))
            .sum::<f64>()
    }).collect();
    let strongest_data = data_strength.iter().copied().fold(0.0f64, f64::max);
    let preferred = preferred_scales(lights);

    let mut scales = preferred.clone();
    for _ in 0..ITERATIONS {
        let mut next = scales.clone();
        for (index, light) in lights.iter().enumerate() {
            let lambda = REGULARIZATION * strongest_data * (1.0 + HAND_BOOST * light.hand);
            let mut numerator = lambda * preferred[index];
            let mut denominator = lambda;
            for (sample, target) in targets.iter().enumerate() {
                let own = contributions[sample][index];
                let own_energy = dot(own, own);
                if own_energy <= EPSILON {
                    continue;
                }
                let mut others = [0.0; 3];
                for other_index in 0..lights.len() {
                    if other_index == index {
                        continue;
                    }
                    let value = contributions[sample][other_index];
                    for channel in 0..3 {
                        others[channel] += scales[other_index] * value[channel];
                    }
                }
                let residual = [
                    target[0] - others[0],
                    target[1] - others[1],
                    target[2] - others[2],
                ];
                numerator += dot(own, residual);
                denominator += own_energy;
            }
            if denominator > EPSILON {
                let optimal = (numerator / denominator).clamp(0.0, 1.0);
                next[index] = scales[index] + DAMPING * (optimal - scales[index]);
            }
        }
        scales = next;
    }

    for (index, light) in lights.iter_mut().enumerate() {
        light.intensity = (f64::from(light.intensity) * scales[index]) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: u32 = 0xffffff;

    fn light(at: [f64; 3], rgb: u32, radius: f32, intensity: f32, hand: bool) -> FieldLight {
        FieldLight::new(at, rgb, radius, intensity, hand)
    }

    fn sum(lights: &[FieldLight], at: [f64; 3]) -> [f64; 3] {
        lights.iter().fold([0.0; 3], |total, light| {
            let value = light.contribution(at);
            [
                total[0] + value[0],
                total[1] + value[1],
                total[2] + value[2],
            ]
        })
    }

    #[test]
    fn coincident_lights_share_one_light_level() {
        let mut lights = vec![
            light([0.0, 64.0, 0.0], WHITE, 10.0, 1.0, false),
            light([0.0, 64.0, 1.0], WHITE, 10.0, 1.0, false),
        ];
        balance(&mut lights);

        assert!(lights.iter().all(|light| light.intensity > 0.1));
        let at_center = sum(&lights, [0.0, 64.0, 0.0]);
        assert!((at_center[0] - 1.0).abs() < 0.05);
    }

    #[test]
    fn separated_lights_keep_their_full_strength() {
        let mut lights = vec![
            light([0.0, 64.0, 0.0], WHITE, 4.0, 1.0, false),
            light([0.0, 64.0, 10.0], WHITE, 4.0, 1.0, false),
        ];
        balance(&mut lights);

        assert!((lights[0].intensity - 1.0).abs() < 1e-3);
        assert!((lights[1].intensity - 1.0).abs() < 1e-3);
    }

    #[test]
    fn an_evenly_matched_hand_keeps_its_strength() {
        let mut lights = vec![
            light([0.0, 64.0, 0.0], WHITE, 10.0, 1.0, true),
            light([0.0, 64.0, 1.0], WHITE, 10.0, 1.0, false),
            light([0.0, 64.0, 2.0], WHITE, 10.0, 1.0, false),
        ];
        balance(&mut lights);

        assert!((lights[0].intensity - 1.0).abs() < 0.05);
        assert!(lights[1].intensity < 0.5);
        assert!(lights[2].intensity < 0.5);
    }

    #[test]
    fn a_weaker_hand_does_not_flatten_a_brighter_neighbour() {
        let mut lights = vec![
            light([0.0, 64.0, 0.0], WHITE, 10.0, 0.4, true),
            light([0.0, 64.0, 1.0], WHITE, 15.0, 1.0, false),
        ];
        balance(&mut lights);

        assert!(lights[1].intensity > 0.9);
        assert!(lights[0].intensity < 0.4);
    }

    #[test]
    fn colours_and_radii_are_left_to_the_engine() {
        let mut lights = vec![
            light([0.0, 64.0, 0.0], 0xff0000, 11.0, 1.0, false),
            light([0.0, 64.0, 1.0], 0x0000ff, 9.0, 1.0, false),
        ];
        balance(&mut lights);

        assert_eq!(lights[0].rgb, 0xff0000);
        assert_eq!(lights[1].rgb, 0x0000ff);
        assert_eq!(lights[0].radius, 11.0);
        assert_eq!(lights[1].radius, 9.0);
        let at_center = sum(&lights, [0.0, 64.0, 0.0]);
        assert!(at_center[0] <= 1.0 + 1e-3);
        assert!(at_center[2] <= 1.0 + 1e-3);
    }
}
