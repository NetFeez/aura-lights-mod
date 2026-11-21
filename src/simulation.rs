//! Test-only simulator for the coloured-light field.
//!
//! It reproduces the engine's WGSL (`sum(color * intensity * falloff)`, clamped to 1) and the
//! Minecraft-style target (`max` per channel), then checks how the balancer behaves in the cases
//! that produced the reported flicker. Run `cargo test simulation -- --nocapture` for the table.

use crate::light_field::{balance, FieldLight};

const TORCH: (u32, f32, f32) = (0xf5ac7b, 14.0, 0.70);
const GLOWSTONE: (u32, f32, f32) = (0xffffff, 15.0, 0.70);
const HALO: (u32, f32, f32) = (0xb358ef, 3.5, 0.22);

#[derive(Clone)]
struct SceneLight {
    label: &'static str,
    at: [f64; 3],
    rgb: u32,
    radius: f32,
    intensity: f32,
    hand: bool,
}

impl SceneLight {
    fn torch(label: &'static str, at: [f64; 3]) -> Self {
        Self::new(label, at, TORCH, false)
    }

    fn new(label: &'static str, at: [f64; 3], spec: (u32, f32, f32), hand: bool) -> Self {
        Self { label, at, rgb: spec.0, radius: spec.1, intensity: spec.2, hand }
    }

    fn field(&self) -> FieldLight {
        FieldLight::new(self.at, self.rgb, self.radius, self.intensity, self.hand)
    }
}

fn at(x: f64, z: f64) -> [f64; 3] {
    [x, 64.0, z]
}

fn color(rgb: u32) -> [f64; 3] {
    [
        f64::from((rgb >> 16) & 0xff) / 255.0,
        f64::from((rgb >> 8) & 0xff) / 255.0,
        f64::from(rgb & 0xff) / 255.0,
    ]
}

fn contribution(light: &FieldLight, point: [f64; 3]) -> [f64; 3] {
    if light.radius <= 0.0 || light.intensity <= 0.0 {
        return [0.0; 3];
    }
    let distance = light.at
        .iter()
        .zip(point)
        .map(|(light, point)| (light - point).powi(2))
        .sum::<f64>()
        .sqrt();
    let falloff = (1.0 - distance / f64::from(light.radius)).max(0.0);
    let strength = falloff * f64::from(light.intensity);
    let rgb = color(light.rgb);
    [rgb[0] * strength, rgb[1] * strength, rgb[2] * strength]
}

/// What the engine actually paints, including its `min(rgb, 1)` clamp.
fn engine(lights: &[FieldLight], point: [f64; 3]) -> [f64; 3] {
    let mut rgb = [0.0f64; 3];
    for light in lights {
        let value = contribution(light, point);
        for channel in 0..3 {
            rgb[channel] += value[channel];
        }
    }
    rgb.map(|channel| channel.min(1.0))
}

/// Minecraft-style block light: the brightest source per channel wins.
fn target(lights: &[FieldLight], point: [f64; 3]) -> [f64; 3] {
    let mut rgb = [0.0f64; 3];
    for light in lights {
        let value = contribution(light, point);
        for channel in 0..3 {
            rgb[channel] = rgb[channel].max(value[channel]);
        }
    }
    rgb.map(|channel| channel.min(1.0))
}

fn grid() -> impl Iterator<Item = [f64; 3]> {
    let mut positions = Vec::new();
    for x in -18..=18 {
        for z in -18..=18 {
            positions.push(at(x as f64, z as f64));
        }
    }
    positions.into_iter()
}

fn error(lights: &[FieldLight], want: &[FieldLight]) -> (f64, f64) {
    let mut total = 0.0;
    let mut count = 0.0;
    let mut worst = 0.0f64;
    for point in grid() {
        let got = engine(lights, point);
        let want = target(want, point);
        for channel in 0..3 {
            let delta = (got[channel] - want[channel]).abs();
            total += delta;
            worst = worst.max(delta);
            count += 1.0;
        }
    }
    (total / count, worst)
}

fn balanced(scene: &[SceneLight]) -> Vec<FieldLight> {
    let mut lights: Vec<_> = scene.iter().map(SceneLight::field).collect();
    balance(&mut lights);
    lights
}

fn intensity<'a>(scene: &'a [SceneLight], lights: &'a [FieldLight], label: &str) -> f32 {
    let index = scene.iter().position(|light| light.label == label).expect(label);
    lights[index].intensity
}

fn report(name: &str, scene: &[SceneLight], lights: &[FieldLight]) {
    let raw: Vec<_> = scene.iter().map(SceneLight::field).collect();
    let (before, _) = error(&raw, &raw);
    let (after, worst) = error(lights, &raw);
    let strengths: Vec<_> = scene.iter()
        .map(|light| format!("{}={:.3}", light.label, intensity(scene, lights, light.label)))
        .collect();
    println!("{name:<22} raw {before:.4} -> fit {after:.4} (max {worst:.3})  {}", strengths.join(" "));
}

fn scenarios() -> Vec<(&'static str, Vec<SceneLight>)> {
    vec![
        ("A only", vec![SceneLight::torch("A", at(0.0, 0.0))]),
        ("A-B", vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::torch("B", at(0.0, 1.0)),
        ]),
        ("AB coincident", vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::torch("B", at(0.0, 0.0)),
        ]),
        ("A-B + halo", vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::torch("B", at(0.0, 1.0)),
            SceneLight::new("P", at(0.0, 0.5), HALO, false),
        ]),
        ("A-B + held torch", vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::torch("B", at(0.0, 1.0)),
            SceneLight::torch("P", at(0.0, 0.5)).as_hand(),
        ]),
        ("weak hand + glowstone", vec![
            SceneLight::new("A", at(0.0, 0.0), GLOWSTONE, false),
            SceneLight::new("P", at(0.0, 1.0), (0x5ae0ea, 10.0, 0.70), true),
        ]),
        ("frame + halo", vec![
            SceneLight::new("A", at(0.0, 0.0), GLOWSTONE, false),
            SceneLight::new("P", at(0.0, 1.0), HALO, false),
        ]),
        ("r11 red + r9 blue", vec![
            SceneLight::new("A", at(0.0, 0.0), (0xff0000, 11.0, 0.70), false),
            SceneLight::new("B", at(0.0, 1.0), (0x0000ff, 9.0, 0.70), false),
        ]),
        ("A-B-C", vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::torch("B", at(0.0, 1.0)),
            SceneLight::torch("C", at(0.0, 2.0)),
        ]),
        ("separate clusters", vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::new("P", at(0.0, 25.0), HALO, false),
        ]),
    ]
}

impl SceneLight {
    fn as_hand(mut self) -> Self {
        self.hand = true;
        self
    }
}

#[test]
fn simulation_report() {
    println!("\n{:22} {:>22} {:>22}", "scenario", "additive error", "balanced error");
    for (name, scene) in scenarios() {
        let lights = balanced(&scene);
        report(name, &scene, &lights);
    }
}

#[test]
fn a_weak_halo_does_not_dim_nearby_lights() {
    let scene = vec![
        SceneLight::torch("A", at(0.0, 0.0)),
        SceneLight::torch("B", at(0.0, 1.0)),
    ];
    let without = balanced(&scene);
    let base = intensity(&scene, &without, "A");

    let mut with_halo = scene.clone();
    with_halo.push(SceneLight::new("P", at(0.0, 0.5), HALO, false));
    let with = balanced(&with_halo);
    let after = intensity(&scene, &with, "A");

    println!("\nA {base:.3} -> {after:.3} when an unlit enchanted player walks in");
    assert!((after - base).abs() < 0.05, "a faint halo must not dim an existing light");
    assert!(intensity(&with_halo, &with, "B") > 0.2);
    assert!(intensity(&with_halo, &with, "P") < 0.3, "the weak halo should fade out");
}

#[test]
fn a_real_held_light_keeps_its_full_strength() {
    let scene = vec![
        SceneLight::torch("A", at(0.0, 0.0)),
        SceneLight::torch("P", at(0.0, 0.5)).as_hand(),
    ];
    let lights = balanced(&scene);

    assert!(intensity(&scene, &lights, "P") > 0.69);
    assert!(intensity(&scene, &lights, "A") < 0.70);
}

#[test]
fn a_weaker_held_light_cannot_flatten_a_brighter_frame() {
    let scene = vec![
        SceneLight::new("A", at(0.0, 0.0), GLOWSTONE, false),
        SceneLight::new("P", at(0.0, 1.0), (0x5ae0ea, 10.0, 0.70), true),
    ];
    let lights = balanced(&scene);

    assert!(intensity(&scene, &lights, "A") > 0.6);
    assert!(intensity(&scene, &lights, "P") < 0.4);
}

#[test]
fn comparable_lights_all_emit_with_their_own_radius_and_color() {
    let scene = vec![
        SceneLight::new("A", at(0.0, 0.0), (0xff0000, 11.0, 0.70), false),
        SceneLight::new("B", at(0.0, 1.0), (0x0000ff, 9.0, 0.70), false),
    ];
    let lights = balanced(&scene);

    assert!(intensity(&scene, &lights, "A") > 0.2);
    assert!(intensity(&scene, &lights, "B") > 0.2);
    assert_eq!(lights[0].rgb, 0xff0000);
    assert_eq!(lights[1].rgb, 0x0000ff);
    assert_eq!(lights[0].radius, 11.0);
    assert_eq!(lights[1].radius, 9.0);
}

#[test]
fn a_far_light_is_not_suppressed_by_a_strong_cluster() {
    let scene = vec![
        SceneLight::torch("A", at(0.0, 0.0)),
        SceneLight::new("P", at(0.0, 25.0), HALO, false),
    ];
    let lights = balanced(&scene);

    assert_eq!(intensity(&scene, &lights, "A"), 0.70);
    assert_eq!(intensity(&scene, &lights, "P"), 0.22);
}

#[test]
fn walking_past_a_weak_halo_changes_the_field_gradually() {
    let mut previous: Option<Vec<FieldLight>> = None;
    let mut worst_step = 0.0f64;
    let probe = at(0.0, 0.0);

    for step in (0..=16).rev() {
        let scene = vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::new("P", at(0.0, step as f64 * 0.5), HALO, false),
        ];
        let lights = balanced(&scene);
        if let Some(previous) = &previous {
            let before = engine(previous, probe);
            let after = engine(&lights, probe);
            worst_step = worst_step
                .max((after[0] - before[0]).abs())
                .max((after[1] - before[1]).abs())
                .max((after[2] - before[2]).abs());
        }
        previous = Some(lights);
    }

    println!("\nlargest field change per 0.5 block step: {worst_step:.4}");
    assert!(worst_step < 0.04, "a moving halo must not make the frame pulse");
}

#[test]
fn walking_in_with_a_real_torch_changes_the_field_gradually() {
    let mut previous: Option<Vec<FieldLight>> = None;
    let mut worst_step = 0.0f64;
    let probe = at(0.0, 0.5);

    for step in (1..=16).rev() {
        let scene = vec![
            SceneLight::torch("A", at(0.0, 0.0)),
            SceneLight::torch("B", at(0.0, 1.0)),
            SceneLight::torch("P", at(0.0, step as f64 * 0.5)).as_hand(),
        ];
        let lights = balanced(&scene);
        if let Some(previous) = &previous {
            let before = engine(previous, probe);
            let after = engine(&lights, probe);
            worst_step = worst_step
                .max((after[0] - before[0]).abs())
                .max((after[1] - before[1]).abs())
                .max((after[2] - before[2]).abs());
        }
        previous = Some(lights);
    }

    println!("\nlargest field change per 0.5 block step with a held torch: {worst_step:.4}");
    assert!(worst_step < 0.06, "the player's light must not make frames pulse");
}
