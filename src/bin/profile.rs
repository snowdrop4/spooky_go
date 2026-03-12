use rand::rngs::SmallRng;
use rand::seq::IndexedRandom;
use rand::SeedableRng;
use spooky_go::bitboard::nw_for_board;
use spooky_go::game::Game;
use spooky_go::outcome::GameOutcome;
use spooky_go::player::Player;

#[hotpath::measure]
fn play_random_game(rng: &mut SmallRng) -> GameOutcome {
    let mut game = Game::<{ nw_for_board(9, 9) }>::new(9, 9);

    loop {
        if let Some(outcome) = game.outcome() {
            return outcome;
        }

        if game.is_over() {
            return game.outcome().unwrap_or(GameOutcome::Draw);
        }

        let moves = game.legal_moves();
        let mv = moves
            .choose(rng)
            .expect("play_random_game: legal moves list must not be empty");
        game.make_move(mv);
    }
}

#[hotpath::main(limit = 0)]
fn main() {
    let num_games = 200;
    let mut black_wins = 0;
    let mut white_wins = 0;
    let mut draws = 0;

    let mut rng = SmallRng::seed_from_u64(0xDEAD_BEEF);

    for _i in 0..num_games {
        let outcome = play_random_game(&mut rng);

        match outcome.winner() {
            Some(Player::Black) => black_wins += 1,
            Some(Player::White) => white_wins += 1,
            None => draws += 1,
        }
    }

    println!("\nResults after {} games:", num_games);
    println!(
        "  Black wins: {} ({:.1}%)",
        black_wins,
        black_wins as f64 / num_games as f64 * 100.0
    );
    println!(
        "  White wins: {} ({:.1}%)",
        white_wins,
        white_wins as f64 / num_games as f64 * 100.0
    );
    println!(
        "  Draws:      {} ({:.1}%)",
        draws,
        draws as f64 / num_games as f64 * 100.0
    );
}
