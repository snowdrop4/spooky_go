import pytest

from spooky_go import Game, Move


class TestMoveCreation:
    def test_place_move(self) -> None:
        move = Move.place(4, 5)
        assert move.col() == 4
        assert move.row() == 5
        assert not move.is_pass()

    def test_pass_move(self) -> None:
        move = Move.pass_move()
        assert move.is_pass()
        assert move.col() is None
        assert move.row() is None


class TestMoveEquality:
    def test_place_moves_equal(self) -> None:
        move1 = Move.place(4, 5)
        move2 = Move.place(4, 5)
        assert move1 == move2

    def test_place_moves_not_equal(self) -> None:
        move1 = Move.place(4, 5)
        move2 = Move.place(5, 4)
        assert move1 != move2

    def test_pass_moves_equal(self) -> None:
        move1 = Move.pass_move()
        move2 = Move.pass_move()
        assert move1 == move2

    def test_place_not_equal_pass(self) -> None:
        place = Move.place(4, 5)
        pass_move = Move.pass_move()
        assert place != pass_move


class TestMoveHash:
    def test_hash_place(self) -> None:
        move = Move.place(4, 5)
        h = hash(move)
        assert isinstance(h, int)

    def test_hash_pass(self) -> None:
        move = Move.pass_move()
        h = hash(move)
        assert isinstance(h, int)

    def test_equal_moves_same_hash(self) -> None:
        move1 = Move.place(4, 5)
        move2 = Move.place(4, 5)
        assert hash(move1) == hash(move2)

    def test_pass_moves_same_hash(self) -> None:
        move1 = Move.pass_move()
        move2 = Move.pass_move()
        assert hash(move1) == hash(move2)


class TestMoveEncoding:
    def test_encode_place_move(self) -> None:
        move = Move.place(0, 0)
        encoded = move.encode(9, 9)
        assert encoded == 0

    def test_encode_place_move_position(self) -> None:
        # row * width + col
        move = Move.place(3, 2)
        encoded = move.encode(9, 9)
        assert encoded == 2 * 9 + 3  # = 21

    def test_encode_pass_move(self) -> None:
        move = Move.pass_move()
        encoded = move.encode(9, 9)
        assert encoded == 81  # width * height

    def test_decode_place_move(self) -> None:
        move = Move.decode(0, 9, 9)
        assert move.col() == 0
        assert move.row() == 0
        assert not move.is_pass()

    def test_decode_place_move_position(self) -> None:
        # action 21 = row 2, col 3 for 9x9 board
        move = Move.decode(21, 9, 9)
        assert move.col() == 3
        assert move.row() == 2

    def test_decode_pass_move(self) -> None:
        move = Move.decode(81, 9, 9)
        assert move.is_pass()

    def test_decode_invalid_action(self) -> None:
        with pytest.raises(ValueError):
            Move.decode(100, 9, 9)

    def test_encode_decode_roundtrip_place(self) -> None:
        for row in range(9):
            for col in range(9):
                move = Move.place(col, row)
                encoded = move.encode(9, 9)
                decoded = Move.decode(encoded, 9, 9)
                assert decoded == move

    def test_encode_decode_roundtrip_pass(self) -> None:
        move = Move.pass_move()
        encoded = move.encode(9, 9)
        decoded = Move.decode(encoded, 9, 9)
        assert decoded == move

    def test_encode_different_board_sizes(self) -> None:
        move = Move.pass_move()

        encoded_9x9 = move.encode(9, 9)
        assert encoded_9x9 == 81

        encoded_19x19 = move.encode(19, 19)
        assert encoded_19x19 == 361


class TestMoveDisplay:
    def test_str_place(self) -> None:
        move = Move.place(4, 5)
        s = str(move)
        assert isinstance(s, str)
        assert "4" in s or "5" in s

    def test_str_pass(self) -> None:
        move = Move.pass_move()
        s = str(move)
        assert isinstance(s, str)
        assert "Pass" in s or "pass" in s

    def test_repr_place(self) -> None:
        move = Move.place(4, 5)
        r = repr(move)
        assert "Move" in r
        assert "place" in r

    def test_repr_pass(self) -> None:
        move = Move.pass_move()
        r = repr(move)
        assert "Move" in r
        assert "pass" in r


class TestMoveInGame:
    def test_moves_from_legal_moves(self) -> None:
        game = Game(9, 9)
        moves = game.legal_moves()

        # Check we have the right number
        assert len(moves) == 82

        # Check pass is in there
        pass_moves = [m for m in moves if m.is_pass()]
        assert len(pass_moves) == 1

        # Check place moves
        place_moves = [m for m in moves if not m.is_pass()]
        assert len(place_moves) == 81
