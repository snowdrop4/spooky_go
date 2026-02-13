use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use spooky_go::encode::encode_game_planes;
use spooky_go::game::Game;
use std::hint::black_box;

/// Play ~20 random moves on a fresh game to create a realistic mid-game position.
/// Uses a fixed seed for reproducibility across benchmark runs.
fn setup_midgame(width: usize, height: usize) -> Game {
    let mut game = Game::new(width, height);
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..20 {
        let moves = game.legal_moves();
        // Filter to placement moves only so the board fills up
        let placements: Vec<_> = moves.iter().filter(|m| !m.is_pass()).copied().collect();
        if placements.is_empty() {
            break;
        }
        let mv = placements.choose(&mut rng).unwrap();
        game.make_move(mv);
    }
    game
}

// ---------------------------------------------------------------------------
// Microbenchmarks
// ---------------------------------------------------------------------------

fn bench_legal_moves_9x9(c: &mut Criterion) {
    let game = setup_midgame(9, 9);
    c.bench_function("legal_moves_9x9", |b| {
        b.iter(|| black_box(game.legal_moves()))
    });
}

fn bench_legal_moves_19x19(c: &mut Criterion) {
    let game = setup_midgame(19, 19);
    c.bench_function("legal_moves_19x19", |b| {
        b.iter(|| black_box(game.legal_moves()))
    });
}

fn bench_make_move(c: &mut Criterion) {
    let game = setup_midgame(9, 9);
    // Pick the first legal placement move
    let moves = game.legal_moves();
    let mv = moves.iter().find(|m| !m.is_pass()).copied().unwrap();
    c.bench_function("make_move", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                black_box(g.make_move(&mv));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_make_unmake(c: &mut Criterion) {
    let game = setup_midgame(9, 9);
    let moves = game.legal_moves();
    let mv = moves.iter().find(|m| !m.is_pass()).copied().unwrap();
    c.bench_function("make_unmake", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                g.make_move(&mv);
                black_box(g.unmake_move());
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_encode_game_planes_9x9(c: &mut Criterion) {
    let game = setup_midgame(9, 9);
    c.bench_function("encode_game_planes_9x9", |b| {
        b.iter(|| black_box(encode_game_planes(&game)))
    });
}

fn bench_encode_game_planes_19x19(c: &mut Criterion) {
    let game = setup_midgame(19, 19);
    c.bench_function("encode_game_planes_19x19", |b| {
        b.iter(|| black_box(encode_game_planes(&game)))
    });
}

fn bench_score(c: &mut Criterion) {
    let game = setup_midgame(9, 9);
    c.bench_function("score", |b| b.iter(|| black_box(game.score())));
}

// ---------------------------------------------------------------------------
// Integration benchmarks
// ---------------------------------------------------------------------------

fn bench_random_playout_9x9(c: &mut Criterion) {
    c.bench_function("random_playout_9x9", |b| {
        b.iter(|| {
            let mut game = Game::new(9, 9);
            let mut rng = StdRng::seed_from_u64(123);
            while !game.is_over() {
                let moves = game.legal_moves();
                let mv = moves.choose(&mut rng).unwrap();
                game.make_move(mv);
            }
            black_box(game.score())
        })
    });
}

fn bench_random_playout_19x19(c: &mut Criterion) {
    c.bench_function("random_playout_19x19", |b| {
        b.iter(|| {
            let mut game = Game::new(19, 19);
            let mut rng = StdRng::seed_from_u64(123);
            while !game.is_over() {
                let moves = game.legal_moves();
                let mv = moves.choose(&mut rng).unwrap();
                game.make_move(mv);
            }
            black_box(game.score())
        })
    });
}

fn bench_self_play_step(c: &mut Criterion) {
    let game = setup_midgame(9, 9);
    c.bench_function("self_play_step", |b| {
        b.iter_batched(
            || game.clone(),
            |mut g| {
                let moves = g.legal_moves();
                let _planes = encode_game_planes(&g);
                // Pick the first legal placement (simulating a policy choice)
                let mv = moves.iter().find(|m| !m.is_pass()).copied().unwrap();
                g.make_move(&mv);
                black_box(&g);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_legal_moves_9x9,
    bench_legal_moves_19x19,
    bench_make_move,
    bench_make_unmake,
    bench_encode_game_planes_9x9,
    bench_encode_game_planes_19x19,
    bench_score,
    bench_random_playout_9x9,
    bench_random_playout_19x19,
    bench_self_play_step,
);
criterion_main!(benches);
