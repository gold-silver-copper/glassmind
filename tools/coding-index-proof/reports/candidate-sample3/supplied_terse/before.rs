/// The player at a 1-based rank, or None if the rank is out of range.
pub fn player_at_rank(players: &[&str], rank: i32) -> Option<String> {
    if rank < 1 || rank as usize > players.len() {
        return None;
    }
    let index = rank - 1;
    Some(players[index].to_string())
}
