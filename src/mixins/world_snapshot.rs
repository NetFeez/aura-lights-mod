fn world_snapshot(game: &Game<'_>) {
    mod_aurora_lights::native::set_dropped(
        game.dropped.iter().map(|item| (
            item.position.components(),
            &*item.stack.item,
            !item.stack.enchantments().is_empty(),
        )),
    );

    // Slots 0 and 1 are MAINHAND and OFFHAND. Remote players live in `mobs` too.
    mod_aurora_lights::native::set_mobs(
        game.mobs.iter()
            .filter(|mob| !mob.removed)
            .map(|mob| (
                mob.id,
                mob.position.components(),
                mob.extra.slot_stack(0).map(|stack| (
                    &*stack.item,
                    !stack.enchantments().is_empty() || horizon_client::mobs::gear::enchanted(mob, 0),
                )),
                mob.extra.slot_stack(1).map(|stack| (
                    &*stack.item,
                    !stack.enchantments().is_empty() || horizon_client::mobs::gear::enchanted(mob, 1),
                )),
                &*mob.kind.id == "minecraft:player",
            )),
    );

    mod_aurora_lights::native::set_frames(
        game.item_frames.iter().map(|frame| (
            frame.pos,
            frame.item.as_ref().map(|stack| (&*stack.item, !stack.enchantments().is_empty())),
            frame.step(),
        )),
    );
}
