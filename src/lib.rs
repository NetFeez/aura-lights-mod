use mod_api::{
    CameraPoseEvent, ClientHost, Hooks, RegistrationContext, Registrar, ShaderParamsEvent,
    VisualsEvent,
};

use crate::glow::emit_held;
use crate::lights::{flush, set_pose};
use crate::settings::light_colors;
use crate::sources::emit as emit_sources;

mod enchant;
mod glow;
mod light_field;
mod lights;
pub mod native;
mod profiles;
mod settings;
#[cfg(test)]
mod simulation;
mod sources;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        settings::register(r);
    }

    fn camera_pose(&self, _host: &dyn ClientHost, event: &mut CameraPoseEvent<'_>) {
        set_pose(*event.position, event.yaw, event.pitch);
    }

    fn visuals(&self, host: &dyn ClientHost, event: &mut VisualsEvent<'_>) {
        let colors = light_colors(host);
        let player_at = host
            .local_player()
            .and_then(|player| player.entity().position(host));
        let mut candidates = Vec::new();
        emit_held(host, &mut candidates, colors);
        emit_sources(host, &mut candidates, colors);
        enchant::emit(host, &mut candidates, colors);
        flush(event, candidates, player_at);
    }

    fn shader_params(&self, host: &dyn ClientHost, event: &mut ShaderParamsEvent<'_>) {
        let colors = if light_colors(host) {
            1.0
        } else {
            0.0
        };
        event.params[0][0] = colors;
    }
}
