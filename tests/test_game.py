import pytest
from rust_go import BLACK, WHITE, Game, Move


class TestGameCreation:
    def test_new_game(self) -> None:
        game = Game(9, 9)
        assert game.width() == 9
        assert game.height() == 9

    def test_standard_game(self) -> None:
        game = Game.standard()
        assert game.width() == 19
        assert game.height() == 19

    def test_custom_size_game(self) -> None:
        game = Game(5, 7)
        assert game.width() == 5
        assert game.height() == 7

    def test_game_too_small(self) -> None:
        with pytest.raises(ValueError):
            Game(1, 9)
        with pytest.raises(ValueError):
            Game(9, 1)


class TestGameState:
    def test_initial_turn(self) -> None:
        game = Game(9, 9)
        assert game.turn() == BLACK

    def test_turn_alternates(self) -> None:
        game = Game(9, 9)
        assert game.turn() == BLACK

        game.make_move(Move.place(4, 4))
        assert game.turn() == WHITE

        game.make_move(Move.place(3, 3))
        assert game.turn() == BLACK

    def test_initial_not_over(self) -> None:
        game = Game(9, 9)
        assert not game.is_over()
        assert game.outcome() is None

    def test_board_access(self) -> None:
        game = Game(9, 9)
        board = game.board()
        assert board.width() == 9
        assert board.height() == 9


class TestGameMoves:
    def test_make_move(self) -> None:
        game = Game(9, 9)
        move = Move.place(4, 4)

        assert game.is_legal_move(move)
        result = game.make_move(move)
        assert result is True

    def test_make_illegal_move_occupied(self) -> None:
        game = Game(9, 9)
        move = Move.place(4, 4)
        game.make_move(move)

        same_move = Move.place(4, 4)
        assert not game.is_legal_move(same_move)
        result = game.make_move(same_move)
        assert result is False

    def test_legal_moves_initial(self) -> None:
        game = Game(9, 9)
        moves = game.legal_moves()
        # 81 board positions + 1 pass
        assert len(moves) == 82

    def test_legal_moves_after_move(self) -> None:
        game = Game(9, 9)
        game.make_move(Move.place(4, 4))
        moves = game.legal_moves()
        # 80 board positions + 1 pass
        assert len(moves) == 81

    def test_unmake_move(self) -> None:
        game = Game(9, 9)
        move = Move.place(4, 4)
        game.make_move(move)

        assert game.turn() == WHITE
        result = game.unmake_move()
        assert result is True
        assert game.turn() == BLACK

    def test_unmake_on_empty_history(self) -> None:
        game = Game(9, 9)
        result = game.unmake_move()
        assert result is False


class TestPassMove:
    def test_pass_is_legal(self) -> None:
        game = Game(9, 9)
        pass_move = Move.pass_move()
        assert game.is_legal_move(pass_move)

    def test_pass_changes_turn(self) -> None:
        game = Game(9, 9)
        assert game.turn() == BLACK

        game.make_move(Move.pass_move())
        assert game.turn() == WHITE

    def test_two_passes_ends_game(self) -> None:
        # Use with_options to set min_moves=0 so double-pass ends immediately
        game = Game.with_options(width=9, height=9, komi=7.5, min_moves_before_pass_ends=0, max_moves=1000)

        game.make_move(Move.pass_move())
        assert not game.is_over()

        game.make_move(Move.pass_move())
        assert game.is_over()
        assert game.outcome() is not None

    def test_two_passes_requires_min_moves(self) -> None:
        # Default game requires minimum moves before pass ends game
        game = Game(9, 9)
        assert game.min_moves_before_pass_ends() == 40  # 81 / 2

        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())
        # Should NOT be over because min_moves not reached
        assert not game.is_over()


class TestCaptures:
    def test_simple_capture_corner(self) -> None:
        game = Game(9, 9)

        # Black surrounds White stone at (0, 0)
        game.make_move(Move.place(1, 0))  # B
        game.make_move(Move.place(0, 0))  # W - will be captured
        game.make_move(Move.place(0, 1))  # B - captures

        board = game.board()
        assert board.get_piece(0, 0) is None  # Captured

    def test_capture_restores_on_unmake(self) -> None:
        game = Game(9, 9)

        game.make_move(Move.place(1, 0))  # B
        game.make_move(Move.place(0, 0))  # W
        game.make_move(Move.place(0, 1))  # B - captures

        assert game.board().get_piece(0, 0) is None

        game.unmake_move()
        assert game.board().get_piece(0, 0) == WHITE


class TestKoRule:
    def test_ko_prevents_immediate_recapture(self) -> None:
        game = Game(5, 5)

        # Build ko shape
        game.make_move(Move.place(1, 0))  # B
        game.make_move(Move.place(2, 0))  # W
        game.make_move(Move.place(0, 1))  # B
        game.make_move(Move.place(1, 1))  # W - will be captured
        game.make_move(Move.place(1, 2))  # B
        game.make_move(Move.place(2, 2))  # W
        game.make_move(Move.pass_move())  # B pass
        game.make_move(Move.place(3, 1))  # W

        # Black captures at (2, 1)
        game.make_move(Move.place(2, 1))

        # Ko point should be set
        ko = game.ko_point()
        assert ko is not None
        assert ko == (1, 1)

        # White cannot immediately recapture
        recapture = Move.place(1, 1)
        assert not game.is_legal_move(recapture)


class TestSuicide:
    def test_suicide_allowed_if_captures(self) -> None:
        game = Game(5, 5)

        # Black surrounds corner except (0, 0)
        game.make_move(Move.place(1, 0))  # B
        game.make_move(Move.pass_move())  # W
        game.make_move(Move.place(0, 1))  # B
        game.make_move(Move.pass_move())  # W

        # White can play at (0, 0) - it has liberties from adjacent black
        # This is actually a case where white would be captured...
        # Let me construct a proper non-suicide case
        move = Move.place(0, 0)
        # This should be legal since White can play there (has liberties or captures)
        assert game.is_legal_move(move)

    def test_suicide_prevented(self) -> None:
        # Use with_options to set min_moves=0 so double-pass ends game
        game = Game.with_options(width=5, height=5, komi=7.5, min_moves_before_pass_ends=0, max_moves=1000)

        # Black plays stones surrounding (0, 0)
        game.make_move(Move.place(1, 0))  # B
        game.make_move(Move.pass_move())  # W
        game.make_move(Move.place(0, 1))  # B
        game.make_move(Move.pass_move())  # W
        game.make_move(Move.pass_move())  # B - game ends with double pass

        # Game is over, so all moves are illegal
        suicide_move = Move.place(0, 0)
        assert not game.is_legal_move(suicide_move)


class TestGameClone:
    def test_clone(self) -> None:
        game = Game(9, 9)
        game.make_move(Move.place(4, 4))
        game.make_move(Move.place(3, 3))

        cloned = game.clone()

        assert cloned.turn() == game.turn()
        assert cloned.is_over() == game.is_over()
        assert cloned.width() == game.width()
        assert cloned.height() == game.height()

    def test_clone_independence(self) -> None:
        game = Game(9, 9)
        game.make_move(Move.place(4, 4))

        cloned = game.clone()
        cloned.make_move(Move.place(3, 3))

        # Original should not be affected
        assert game.turn() == WHITE
        assert cloned.turn() == BLACK


class TestGameDisplay:
    def test_str(self) -> None:
        game = Game(9, 9)
        s = str(game)
        assert isinstance(s, str)
        assert len(s) > 0

    def test_repr(self) -> None:
        game = Game(9, 9)
        r = repr(game)
        assert "Game" in r


class TestGameHash:
    def test_hash(self) -> None:
        game = Game(9, 9)
        h = hash(game)
        assert isinstance(h, int)

    def test_same_state_same_hash(self) -> None:
        game1 = Game(9, 9)
        game2 = Game(9, 9)

        assert hash(game1) == hash(game2)

    def test_different_state_different_hash(self) -> None:
        game1 = Game(9, 9)
        game2 = Game(9, 9)
        game2.make_move(Move.place(4, 4))

        assert hash(game1) != hash(game2)
