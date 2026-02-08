from pathlib import Path
import sys
import time

import spooky_go

# Add dlgo submodule to path
dlgo_path = Path(__file__).parent / "deep_learning_and_the_game_of_go" / "code"
sys.path.insert(0, str(dlgo_path))

from dlgo.goboard import GameState, Move
from dlgo.gotypes import Point


def test_initial_move_generation_speed() -> None:
    iterations = 1000

    # Time spooky_go
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_go.Game(9, 9)
        moves = game.legal_moves()
        assert len(moves) == 82  # 81 positions + pass
    rust_time = time.time() - rust_start

    # Time dlgo
    dlgo_start = time.time()
    for _ in range(iterations):
        game = GameState.new_game(9)
        moves = game.legal_moves()
        assert len(moves) == 83  # 81 positions + pass + resign
    dlgo_time = time.time() - dlgo_start

    print(f"\nMove generation 9x9 ({iterations} iterations):")
    print(f"  spooky_go: {rust_time:.4f}s")
    print(f"  dlgo: {dlgo_time:.4f}s")
    print(f"  Speedup: {dlgo_time / rust_time:.2f}x")


def test_initial_move_generation_speed_19x19() -> None:
    iterations = 100

    # Time spooky_go
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_go.Game(19, 19)
        moves = game.legal_moves()
        assert len(moves) == 362  # 361 positions + pass
    rust_time = time.time() - rust_start

    # Time dlgo
    dlgo_start = time.time()
    for _ in range(iterations):
        game = GameState.new_game(19)
        moves = game.legal_moves()
        assert len(moves) == 363  # 361 positions + pass + resign
    dlgo_time = time.time() - dlgo_start

    print(f"\nMove generation 19x19 ({iterations} iterations):")
    print(f"  spooky_go: {rust_time:.4f}s")
    print(f"  dlgo: {dlgo_time:.4f}s")
    print(f"  Speedup: {dlgo_time / rust_time:.2f}x")


def test_move_making_speed() -> None:
    iterations = 1000

    # Time spooky_go
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_go.Game(9, 9)
        move = spooky_go.Move.place(4, 4)
        game.make_move(move)
    rust_time = time.time() - rust_start

    # Time dlgo (1-indexed)
    dlgo_start = time.time()
    for _ in range(iterations):
        game = GameState.new_game(9)
        move = Move.play(Point(5, 5))  # 1-indexed, so (5,5) is center
        game.apply_move(move)
    dlgo_time = time.time() - dlgo_start

    print(f"\nMove making ({iterations} iterations):")
    print(f"  spooky_go: {rust_time:.4f}s")
    print(f"  dlgo: {dlgo_time:.4f}s")
    print(f"  Speedup: {dlgo_time / rust_time:.2f}x")


def test_move_sequence_speed() -> None:
    iterations = 500

    # A simple sequence of moves (alternating corners and center)
    # spooky_go uses 0-indexed, dlgo uses 1-indexed
    rust_moves_coords = [
        (2, 2),
        (6, 6),
        (2, 6),
        (6, 2),
        (4, 4),
        (3, 3),
        (5, 5),
        (3, 5),
        (5, 3),
        (4, 2),
    ]

    dlgo_moves_coords = [
        (3, 3),
        (7, 7),
        (3, 7),
        (7, 3),
        (5, 5),
        (4, 4),
        (6, 6),
        (4, 6),
        (6, 4),
        (5, 3),
    ]

    # Time spooky_go
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_go.Game(9, 9)
        for col, row in rust_moves_coords:
            move = spooky_go.Move.place(col, row)
            game.make_move(move)
    rust_time = time.time() - rust_start

    # Time dlgo
    dlgo_start = time.time()
    for _ in range(iterations):
        game = GameState.new_game(9)
        for row, col in dlgo_moves_coords:
            move = Move.play(Point(row, col))
            game = game.apply_move(move)
    dlgo_time = time.time() - dlgo_start

    print(f"\nMove sequence ({iterations} iterations, {len(rust_moves_coords)} moves each):")
    print(f"  spooky_go: {rust_time:.4f}s")
    print(f"  dlgo: {dlgo_time:.4f}s")
    print(f"  Speedup: {dlgo_time / rust_time:.2f}x")


def test_game_simulation_speed() -> None:
    def simulate_game_rust(moves_count: int) -> int:
        game = spooky_go.Game(9, 9)
        moves_made = 0

        while moves_made < moves_count and not game.is_over():
            legal_moves = game.legal_moves()
            # Skip pass moves for simulation, pick first place move
            place_moves = [m for m in legal_moves if not m.is_pass()]
            if place_moves:
                game.make_move(place_moves[0])
                moves_made += 1
            else:
                break

        return moves_made

    def simulate_game_dlgo(moves_count: int) -> int:
        game = GameState.new_game(9)
        moves_made = 0

        while moves_made < moves_count and not game.is_over():
            legal_moves = game.legal_moves()
            # Skip pass and resign moves for simulation
            place_moves = [m for m in legal_moves if m.is_play]
            if place_moves:
                game = game.apply_move(place_moves[0])
                moves_made += 1
            else:
                break

        return moves_made

    iterations = 100
    moves_count = 30

    # Time spooky_go
    rust_start = time.time()
    for _ in range(iterations):
        simulate_game_rust(moves_count)
    rust_time = time.time() - rust_start

    # Time dlgo
    dlgo_start = time.time()
    for _ in range(iterations):
        simulate_game_dlgo(moves_count)
    dlgo_time = time.time() - dlgo_start

    print(f"\nGame simulation ({iterations} games, {moves_count} moves each):")
    print(f"  spooky_go: {rust_time:.4f}s")
    print(f"  dlgo: {dlgo_time:.4f}s")
    print(f"  Speedup: {dlgo_time / rust_time:.2f}x")


def test_legal_moves_after_captures_speed() -> None:
    iterations = 500

    # Set up a position with a capture
    # spooky_go: 0-indexed
    rust_setup_moves = [
        (1, 0),  # B
        (0, 0),  # W - will be captured
        (0, 1),  # B - captures W
    ]

    # dlgo: 1-indexed
    dlgo_setup_moves = [
        (1, 2),  # B
        (1, 1),  # W - will be captured
        (2, 1),  # B - captures W
    ]

    # Time spooky_go
    rust_start = time.time()
    for _ in range(iterations):
        game = spooky_go.Game(9, 9)
        for col, row in rust_setup_moves:
            game.make_move(spooky_go.Move.place(col, row))
        _ = game.legal_moves()
    rust_time = time.time() - rust_start

    # Time dlgo
    dlgo_start = time.time()
    for _ in range(iterations):
        game = GameState.new_game(9)
        for row, col in dlgo_setup_moves:
            game = game.apply_move(Move.play(Point(row, col)))
        _ = game.legal_moves()
    dlgo_time = time.time() - dlgo_start

    print(f"\nLegal moves after capture ({iterations} iterations):")
    print(f"  spooky_go: {rust_time:.4f}s")
    print(f"  dlgo: {dlgo_time:.4f}s")
    print(f"  Speedup: {dlgo_time / rust_time:.2f}x")
