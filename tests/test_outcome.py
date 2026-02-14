from spooky_go import BLACK, WHITE, Game, Move


class TestOutcomeFromGame:
    def test_no_outcome_initially(self) -> None:
        game = Game(9, 9)
        assert game.outcome() is None

    def test_outcome_after_two_passes(self) -> None:
        # Use with_options to set min_moves=0 so double-pass ends immediately
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None


class TestOutcomeProperties:
    def test_white_wins_empty_board_with_komi(self) -> None:
        # On empty board, White wins due to komi (7.5 default)
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None
        assert outcome.winner() == WHITE
        assert not outcome.is_draw()

    def test_white_wins_encoding_absolute(self) -> None:
        # On empty board, White wins due to komi
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None
        assert outcome.encode_winner_absolute() == -1.0  # White win

    def test_encoding_from_perspective(self) -> None:
        # On empty board, White wins due to komi
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None
        assert outcome.encode_winner_from_perspective(BLACK) == -1.0  # Loss for Black
        assert outcome.encode_winner_from_perspective(WHITE) == 1.0  # Win for White

    def test_black_wins_with_territory(self) -> None:
        # Create a game where Black has enough territory to overcome komi
        # Use with_options to set min_moves=0 so double-pass ends immediately
        game = Game.with_options(
            width=5,
            height=5,
            komi=0.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )

        # Black plays stones, White passes
        game.make_move(Move.place(0, 0))  # Black
        game.make_move(Move.pass_move())  # White
        game.make_move(Move.place(1, 0))  # Black
        game.make_move(Move.pass_move())  # White
        game.make_move(Move.place(0, 1))  # Black
        game.make_move(Move.pass_move())  # White
        game.make_move(Move.place(1, 1))  # Black
        game.make_move(Move.pass_move())  # White
        game.make_move(Move.pass_move())  # Black - game ends

        outcome = game.outcome()
        assert outcome is not None
        assert outcome.winner() == BLACK
        assert outcome.encode_winner_absolute() == 1.0

    def test_score_method(self) -> None:
        # Test that the score method works
        game = Game.with_options(width=5, height=5, komi=7.5, min_moves_before_pass_ends=0, max_moves=1000, superko=True)
        black_score, white_score = game.score()
        assert black_score == 0.0  # Empty board, no Black stones/territory
        assert white_score == 7.5  # Just komi


class TestOutcomeDisplay:
    def test_str(self) -> None:
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None
        s = str(outcome)
        assert isinstance(s, str)
        assert len(s) > 0

    def test_repr(self) -> None:
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None
        r = repr(outcome)
        assert "GameOutcome" in r

    def test_name(self) -> None:
        game = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game.make_move(Move.pass_move())
        game.make_move(Move.pass_move())

        outcome = game.outcome()
        assert outcome is not None
        name = outcome.name()
        assert isinstance(name, str)


class TestOutcomeEquality:
    def test_same_outcomes_equal(self) -> None:
        game1 = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game1.make_move(Move.pass_move())
        game1.make_move(Move.pass_move())

        game2 = Game.with_options(
            width=9,
            height=9,
            komi=7.5,
            min_moves_before_pass_ends=0,
            max_moves=1000,
        )
        game2.make_move(Move.pass_move())
        game2.make_move(Move.pass_move())

        outcome1 = game1.outcome()
        outcome2 = game2.outcome()

        assert outcome1 is not None
        assert outcome2 is not None
        assert outcome1 == outcome2
