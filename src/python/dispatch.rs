use crate::bitboard::nw_for_board;
use crate::board::Board;
use crate::game::Game;

// -----------------------------------------------------------------------
// Enum dispatch via paste! for Game<NW> and Board<NW>
// -----------------------------------------------------------------------

macro_rules! define_dispatch {
    ($($nw:literal),*) => {
        paste::paste! {
            #[derive(Clone, Debug)]
            pub(super) enum GameInner {
                $( [<Nw $nw>](Game<$nw>), )*
            }

            #[derive(Clone, Debug)]
            pub(super) enum BoardInner {
                $( [<Nw $nw>](Board<$nw>), )*
            }

            macro_rules! dispatch_game {
                ($self_:expr, $g:ident => $body:expr) => {
                    match $self_ {
                        $( GameInner::[<Nw $nw>]($g) => $body, )*
                    }
                };
            }

            macro_rules! dispatch_game_mut {
                ($self_:expr, $g:ident => $body:expr) => {
                    match $self_ {
                        $( GameInner::[<Nw $nw>]($g) => $body, )*
                    }
                };
            }

            macro_rules! dispatch_board {
                ($self_:expr, $b:ident => $body:expr) => {
                    match $self_ {
                        $( BoardInner::[<Nw $nw>]($b) => $body, )*
                    }
                };
            }

            macro_rules! dispatch_board_mut {
                ($self_:expr, $b:ident => $body:expr) => {
                    match $self_ {
                        $( BoardInner::[<Nw $nw>]($b) => $body, )*
                    }
                };
            }

            pub(super) fn make_game_inner(width: u8, height: u8) -> GameInner {
                let nw = nw_for_board(width, height);
                match nw {
                    $( $nw => GameInner::[<Nw $nw>](Game::new(width, height)), )*
                    _ => unreachable!("NW out of range: {}", nw),
                }
            }

            pub(super) fn make_game_inner_with_options(
                width: u8, height: u8, komi: f32,
                min_moves: u16, max_moves: u16, superko: bool,
            ) -> GameInner {
                let nw = nw_for_board(width, height);
                match nw {
                    $( $nw => GameInner::[<Nw $nw>](Game::with_options(
                        width, height, komi, min_moves, max_moves, superko
                    )), )*
                    _ => unreachable!("NW out of range: {}", nw),
                }
            }

            pub(super) fn make_board_inner(width: u8, height: u8) -> BoardInner {
                let nw = nw_for_board(width, height);
                match nw {
                    $( $nw => BoardInner::[<Nw $nw>](Board::new(width, height)), )*
                    _ => unreachable!("NW out of range: {}", nw),
                }
            }

            macro_rules! game_to_board_inner {
                ($game_inner:expr) => {
                    match $game_inner {
                        $( GameInner::[<Nw $nw>](g) => BoardInner::[<Nw $nw>](*g.board()), )*
                    }
                };
            }
        }
    }
}

define_dispatch!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
