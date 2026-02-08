from rust_go import TOTAL_INPUT_PLANES, Game, Move


def get_plane_value(data: list[float], plane: int, row: int, col: int, height: int, width: int,) -> float:
    return data[plane * height * width + row * width + col]


class TestConstants:
    def test_total_input_planes(self) -> None:
        # Should be (HISTORY_LENGTH * 2) + 1 = 17
        assert TOTAL_INPUT_PLANES == 17


class TestGameEncoding:
    def test_encode_game_planes_shape(self) -> None:
        game = Game(9, 9)
        data, num_planes, height, width = game.encode_game_planes()

        assert num_planes == TOTAL_INPUT_PLANES
        assert height == 9
        assert width == 9
        assert len(data) == num_planes * height * width

    def test_encode_game_planes_empty(self) -> None:
        game = Game(9, 9)
        data, num_planes, height, width = game.encode_game_planes()

        # First 16 planes should be zeros (current, opponent x 8 history)
        for plane_idx in range(16):
            for row in range(height):
                for col in range(width):
                    assert get_plane_value(data, plane_idx, row, col, height, width) == 0.0

        # Last plane is color plane (Black's turn = 1.0)
        for row in range(height):
            for col in range(width):
                assert get_plane_value(data, 16, row, col, height, width) == 1.0

    def test_encode_game_planes_with_pieces(self) -> None:
        game = Game(9, 9)
        game.make_move(Move.place(4, 4))  # Black at (4, 4)
        game.make_move(Move.place(3, 3))  # White at (3, 3)

        # Now it's Black's turn, so perspective is Black
        data, num_planes, height, width = game.encode_game_planes()

        # Plane 0: current player (Black) stones
        # Plane 1: opponent (White) stones

        # Black stone at (4, 4)
        assert get_plane_value(data, 0, 4, 4, height, width) == 1.0

        # White stone at (3, 3)
        assert get_plane_value(data, 1, 3, 3, height, width) == 1.0

    def test_encode_game_planes_color_plane(self) -> None:
        game = Game(9, 9)

        # Black's turn
        data, num_planes, height, width = game.encode_game_planes()
        assert get_plane_value(data, 16, 0, 0, height, width) == 1.0

        # After Black moves, White's turn
        game.make_move(Move.place(4, 4))
        data, num_planes, height, width = game.encode_game_planes()
        assert get_plane_value(data, 16, 0, 0, height, width) == 0.0

    def test_encode_game_planes_different_sizes(self) -> None:
        game_9 = Game(9, 9)
        data_9, num_planes_9, height_9, width_9 = game_9.encode_game_planes()

        assert height_9 == 9
        assert width_9 == 9
        assert len(data_9) == num_planes_9 * 9 * 9

        game_19 = Game(19, 19)
        data_19, num_planes_19, height_19, width_19 = game_19.encode_game_planes()

        assert height_19 == 19
        assert width_19 == 19
        assert len(data_19) == num_planes_19 * 19 * 19


class TestActionDecoding:
    def test_decode_action_place(self) -> None:
        game = Game(9, 9)
        move = game.decode_action(0)

        assert move is not None
        assert move.col() == 0
        assert move.row() == 0

    def test_decode_action_pass(self) -> None:
        game = Game(9, 9)
        move = game.decode_action(81)

        assert move is not None
        assert move.is_pass()

    def test_decode_action_invalid(self) -> None:
        game = Game(9, 9)
        move = game.decode_action(100)
        assert move is None

    def test_total_actions(self) -> None:
        game_9 = Game(9, 9)
        assert game_9.total_actions() == 82  # 81 + pass

        game_19 = Game(19, 19)
        assert game_19.total_actions() == 362  # 361 + pass


class TestEncodingConsistency:
    def test_encoding_deterministic(self) -> None:
        game = Game(9, 9)
        game.make_move(Move.place(4, 4))
        game.make_move(Move.place(3, 3))

        planes1 = game.encode_game_planes()
        planes2 = game.encode_game_planes()

        assert planes1 == planes2

    def test_encoding_after_unmake(self) -> None:
        game = Game(9, 9)
        initial_planes = game.encode_game_planes()

        game.make_move(Move.place(4, 4))
        game.make_move(Move.place(3, 3))
        game.unmake_move()
        game.unmake_move()

        final_planes = game.encode_game_planes()
        assert initial_planes == final_planes

    def test_different_positions_different_encoding(self) -> None:
        game1 = Game(9, 9)
        game1.make_move(Move.place(0, 0))

        game2 = Game(9, 9)
        game2.make_move(Move.place(1, 0))

        planes1 = game1.encode_game_planes()
        planes2 = game2.encode_game_planes()

        assert planes1 != planes2
