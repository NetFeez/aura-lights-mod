use crate::sources;

/// ([x, y, z], item id, has enchantments)
pub fn set_dropped<'a>(items: impl IntoIterator<Item = ([f64; 3], &'a str, bool)>) {
    sources::set_dropped(items);
}

/// (entity id, [x, y, z], main hand, off hand, is a player)
///
/// Each hand is `(item id, has enchantments)`.
pub fn set_mobs<'a>(
    mobs: impl IntoIterator<Item = (u32, [f64; 3], Option<(&'a str, bool)>, Option<(&'a str, bool)>, bool)>,
) {
    sources::set_mobs(mobs);
}

/// (block position, item as `(id, has enchantments)`, face step)
pub fn set_frames<'a>(
    frames: impl IntoIterator<Item = ([i32; 3], Option<(&'a str, bool)>, [i32; 3])>,
) {
    sources::set_frames(frames);
}
