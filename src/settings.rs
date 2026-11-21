use mod_api::{ClientHost, Registrar, SettingSpec};

pub(crate) const OTHER_PLAYERS: &str = "other_players";
pub(crate) const DROPPED_ITEMS: &str = "dropped_items";
pub(crate) const LIGHT_COLORS: &str = "light_colors";
pub(crate) const ENCHANTED_GLOW: &str = "enchanted_glow";
pub(crate) const ITEM_FRAMES: &str = "item_frames";

const ON: &str = "netfeez_aurora_lights.setting.on";
const OFF: &str = "netfeez_aurora_lights.setting.off";
const ON_INDEX: u32 = 0;

const OTHER_PLAYERS_LABEL: &str = "netfeez_aurora_lights.setting.other_players";
const DROPPED_ITEMS_LABEL: &str = "netfeez_aurora_lights.setting.dropped_items";
const LIGHT_COLORS_LABEL: &str = "netfeez_aurora_lights.setting.light_colors";
const ENCHANTED_GLOW_LABEL: &str = "netfeez_aurora_lights.setting.enchanted_glow";
const ITEM_FRAMES_LABEL: &str = "netfeez_aurora_lights.setting.item_frames";

const OTHER_PLAYERS_TOOLTIP: &str = "netfeez_aurora_lights.setting.other_players.tooltip";
const DROPPED_ITEMS_TOOLTIP: &str = "netfeez_aurora_lights.setting.dropped_items.tooltip";
const LIGHT_COLORS_TOOLTIP: &str = "netfeez_aurora_lights.setting.light_colors.tooltip";
const ENCHANTED_GLOW_TOOLTIP: &str = "netfeez_aurora_lights.setting.enchanted_glow.tooltip";
const ITEM_FRAMES_TOOLTIP: &str = "netfeez_aurora_lights.setting.item_frames.tooltip";

pub(crate) fn register(r: &mut dyn Registrar) {
    r.register_setting(toggle(OTHER_PLAYERS, OTHER_PLAYERS_LABEL, OTHER_PLAYERS_TOOLTIP, true))
        .expect("register other_players setting");
    r.register_setting(toggle(DROPPED_ITEMS, DROPPED_ITEMS_LABEL, DROPPED_ITEMS_TOOLTIP, true))
        .expect("register dropped_items setting");
    r.register_setting(toggle(LIGHT_COLORS, LIGHT_COLORS_LABEL, LIGHT_COLORS_TOOLTIP, false))
        .expect("register light_colors setting");
    r.register_setting(toggle(ENCHANTED_GLOW, ENCHANTED_GLOW_LABEL, ENCHANTED_GLOW_TOOLTIP, true))
        .expect("register enchanted_glow setting");
    r.register_setting(toggle(ITEM_FRAMES, ITEM_FRAMES_LABEL, ITEM_FRAMES_TOOLTIP, true))
        .expect("register item_frames setting");
}

pub(crate) fn other_players(host: &dyn ClientHost) -> bool {
    on(host, OTHER_PLAYERS, true)
}

pub(crate) fn dropped_items(host: &dyn ClientHost) -> bool {
    on(host, DROPPED_ITEMS, true)
}

pub(crate) fn light_colors(host: &dyn ClientHost) -> bool {
    on(host, LIGHT_COLORS, false)
}

pub(crate) fn enchanted_glow(host: &dyn ClientHost) -> bool {
    on(host, ENCHANTED_GLOW, true)
}

pub(crate) fn item_frames(host: &dyn ClientHost) -> bool {
    on(host, ITEM_FRAMES, true)
}

fn toggle(key: &str, label: &str, tooltip: &str, default_on: bool) -> SettingSpec {
    let default = if default_on { 0 } else { 1 };
    SettingSpec::choice(key, label, [ON, OFF], default).tooltip(tooltip)
}

fn on(host: &dyn ClientHost, key: &str, default_on: bool) -> bool {
    let fallback = if default_on { ON_INDEX } else { ON_INDEX + 1 };
    host.setting_choice(key).unwrap_or(fallback) == ON_INDEX
}
