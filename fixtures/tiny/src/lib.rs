//! A tiny game, used as a fixture source in CI.

/// How fast something moves.
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

/// The player.
pub struct Player {
    pub hp: u32,
    pub speed: Velocity,
}

/// An enemy.
pub struct Enemy {
    pub strength: u32,
    pub drift: Velocity,
}

/// Lowers the player's hit points.
pub fn apply_damage(player: &mut Player, amount: u32) {
    player.hp = player.hp.saturating_sub(amount);
}

/// Spawns one wave of enemies.
pub fn spawn_wave(level: u32) -> Vec<Enemy> {
    (0..level).map(|i| make_enemy(i)).collect()
}

fn make_enemy(strength: u32) -> Enemy {
    Enemy {
        strength,
        drift: Velocity { dx: 0.0, dy: 1.0 },
    }
}

/// One step of the game.
pub fn tick(player: &mut Player, level: u32) {
    let wave = spawn_wave(level);
    for enemy in &wave {
        apply_damage(player, enemy.strength);
    }
}
