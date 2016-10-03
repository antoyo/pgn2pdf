/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use chess_pgn_parser::{GameMove, Square};
use chess_pgn_parser::Move::{BasicMove, CastleKingside, CastleQueenside};
use chess_pgn_parser::Piece::{self, Bishop, King, Knight, Pawn, Queen, Rook};

use self::Color::{Black, White};

macro_rules! play {
    ($ident:ident, $color:expr, $delta:expr) => {
        fn $ident(&mut self, game_move: &GameMove) {
            // TODO: check if the king is in check to see if the move is legal.
            match game_move.move_.move_ {
                BasicMove { ref from, is_capture, ref piece, ref promoted_to, ref to } => {
                    let (maybe_from_x, maybe_from_y) = square_to_maybe_indexes(from);
                    let (to_x, to_y) = square_to_indexes(to);
                    match *piece {
                        Bishop => {
                            let (from_x, from_y) = self.find_bishop(to_x, to_y, $color, maybe_from_x, maybe_from_y);
                            self.board[to_y][to_x] = Some(($color, Bishop));
                            self.board[from_y][from_x] = None;
                        },
                        Knight => {
                            let (from_x, from_y) = self.find_knight(to_x, to_y, $color, maybe_from_x, maybe_from_y);
                            self.board[to_y][to_x] = Some(($color, Knight));
                            self.board[from_y][from_x] = None;
                        },
                        Pawn => {
                            // TODO: en passant.
                            let (from_x, from_y) = self.find_pawn(to_x, to_y, is_capture, $delta);
                            let new_piece =
                                if let Some(piece) = *promoted_to {
                                    piece
                                }
                                else {
                                    Pawn
                                };
                            self.board[to_y][to_x] = Some(($color, new_piece));
                            self.board[from_y][from_x] = None;
                        },
                        _ => (), // TODO
                    }
                },
                CastleKingside => {
                    let line =
                        match $color {
                            Black => 0,
                            White => 7,
                        };
                    self.board[line][6] = Some(($color, King));
                    self.board[line][4] = None;
                    self.board[line][5] = Some(($color, Rook));
                    self.board[line][7] = None;
                },
                CastleQueenside => {
                    let line =
                        match $color {
                            Black => 0,
                            White => 7,
                        };
                    self.board[line][2] = Some(($color, King));
                    self.board[line][4] = None;
                    self.board[line][3] = Some(($color, Rook));
                    self.board[line][0] = None;
                },
            }
        }
    };
}

#[derive(PartialEq)]
pub enum Color {
    Black,
    White,
}

pub struct ChessGame {
    board: [[Option<(Color, Piece)>; 8]; 8],
    turn: Color,
}

impl ChessGame {
    pub fn initial() -> Self {
        let board = [
            [Some((Black, Rook)), Some((Black, Knight)),Some((Black, Bishop)), Some((Black, Queen)), Some((Black, King)), Some((Black, Bishop)), Some((Black, Knight)), Some((Black, Rook))],
            [Some((Black, Pawn)), Some((Black, Pawn)), Some((Black, Pawn)), Some((Black, Pawn)), Some((Black, Pawn)), Some((Black, Pawn)), Some((Black, Pawn)), Some((Black, Pawn))],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [Some((White, Pawn)), Some((White, Pawn)), Some((White, Pawn)), Some((White, Pawn)), Some((White, Pawn)), Some((White, Pawn)), Some((White, Pawn)), Some((White, Pawn))],
            [Some((White, Rook)), Some((White, Knight)), Some((White, Bishop)), Some((White, Queen)), Some((White, King)), Some((White, Bishop)), Some((White, Knight)), Some((White, Rook))],
        ];
        ChessGame {
            board: board,
            turn: White,
        }
    }

    fn find_bishop(&self, to_x: usize, to_y: usize, color: Color, maybe_from_x: Option<usize>, maybe_from_y: Option<usize>) -> (usize, usize) {
        if let (Some(from_x), Some(from_y)) = (maybe_from_x, maybe_from_y) {
            return (from_x, from_y);
        }
        let deltas = [
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];
        for &(dx, dy) in &deltas {
            let mut x = to_x as i32;
            let mut y = to_y as i32;
            while is_valid(x, y) {
                {
                    let x = x as usize;
                    let y = y as usize;
                    match self.board[y][x] {
                        None => (),
                        Some((ref square_color, Bishop)) => {
                            if *square_color == color {
                                if let Some(from_x) = maybe_from_x {
                                    if x == from_x {
                                        return (x, y);
                                    }
                                }
                                else if let Some(from_y) = maybe_from_y {
                                    if y == from_y {
                                        return (x, y);
                                    }
                                }
                                else {
                                    return (x, y);
                                }
                            }
                        },
                        _ => break,
                    }
                }
                x += dx;
                y += dy;
            }
        }
        unreachable!()
    }

    fn find_knight(&self, to_x: usize, to_y: usize, color: Color, maybe_from_x: Option<usize>, maybe_from_y: Option<usize>) -> (usize, usize) {
        if let (Some(from_x), Some(from_y)) = (maybe_from_x, maybe_from_y) {
            return (from_x, from_y);
        }
        let deltas = [
            (-1, -2),
            (-2, -1),
            (1, -2),
            (2, -1),
            (-1, 2),
            (-2, 1),
            (1, 2),
            (2, 1),
        ];

        for &(dx, dy) in &deltas {
            let x = to_x as i32 + dx;
            let y = to_y as i32 + dy;
            if is_valid(x, y) {
                let x = x as usize;
                let y = y as usize;
                if let Some((ref square_color, Knight)) = self.board[y][x] {
                    if *square_color == color {
                        if let Some(from_x) = maybe_from_x {
                            if x == from_x {
                                return (x, y);
                            }
                        }
                        else if let Some(from_y) = maybe_from_y {
                            if y == from_y {
                                return (x, y);
                            }
                        }
                        else {
                            return (x, y);
                        }
                    }
                }
            }
        }
        unreachable!()
    }

    fn find_pawn(&self, to_x: usize, to_y: usize, is_capture: bool, delta: i32) -> (usize, usize) {
        let index1 = to_y as i32 + delta;
        let index2 = to_y as i32 + delta * 2;
        if coord_valid(index1) {
            let index1 = index1 as usize;
            if is_capture {
                let x1 = to_x as i32 - 1;
                if coord_valid(x1) {
                    let x1 = x1 as usize;
                    if let Some((_, Pawn)) = self.board[index1][x1] {
                        return (x1, index1);
                    }
                }
                let x2 = to_x as i32 + 1;
                if coord_valid(x2) {
                    let x2 = x2 as usize;
                    if let Some((_, Pawn)) = self.board[index1][x2] {
                        return (x2, index1);
                    }
                }
                unreachable!()
            }
            else {
                if let Some((_, Pawn)) = self.board[index1][to_x] {
                    return (to_x, index1);
                }
                if coord_valid(index2) {
                    let index2 = index2 as usize;
                    if let Some((_, Pawn)) = self.board[index2][to_x] {
                        return (to_x, index2);
                    }
                }
            }
        }
        unreachable!()
    }

    pub fn play(&mut self, game_move: &GameMove) {
        if self.turn == White {
            self.play_white(game_move);
            self.turn = Black;
        }
        else {
            self.play_black(game_move);
            self.turn = White;
        }
    }

    play!(play_black, Black, -1);
    play!(play_white, White, 1);

    pub fn show(&self) -> String {
        let mut string = "&#58120;&#58152;&#58153;&#58154;&#58155;&#58156;&#58157;&#58158;&#58159;&#58121; +\n".to_string();
        for (y, row) in self.board.iter().enumerate() {
            let border = 7 - y + 0xE310;
            string.push_str(&format!("&#{};", border));
            for (x, square) in row.iter().enumerate() {
                let white_square = (x + y) % 2 == 0;
                let num = piece_to_num(square, white_square);
                string.push_str(&format!("&#{};", num));
            }
            let border = border + 0x10;
            string.push_str(&format!("&#{};", border));
            if y == 0 && self.turn == Black {
                string.push_str("icon:circle[size=70%]");
            }
            else if y == 7 && self.turn == White {
                string.push_str("icon:circle-thin[size=70%]");
            }
            string.push_str(" +\n");
        }
        string.push_str("&#58122;&#58136;&#58137;&#58138;&#58139;&#58140;&#58141;&#58142;&#58143;&#58123; +");
        string
    }
}

fn piece_to_num(square: &Option<(Color, Piece)>, white_square: bool) -> i32 {
    match *square {
        Some(ref square) => {
            if white_square {
                match *square {
                    (Black, Bishop) => 0x265D,
                    (Black, King) => 0x265A,
                    (Black, Knight) => 0x265E,
                    (Black, Pawn) => 0x265F,
                    (Black, Queen) => 0x265B,
                    (Black, Rook) => 0x265C,
                    (White, Bishop) => 0x2657,
                    (White, King) => 0x2654,
                    (White, Knight) => 0x2658,
                    (White, Pawn) => 0x2659,
                    (White, Queen) => 0x2655,
                    (White, Rook) => 0x2656,
                }
            }
            else {
                match *square {
                    (Black, Bishop) => 0xE15D,
                    (Black, King) => 0xE15A,
                    (Black, Knight) => 0xE15E,
                    (Black, Pawn) => 0xE15F,
                    (Black, Queen) => 0xE15B,
                    (Black, Rook) => 0xE15C,
                    (White, Bishop) => 0xE157,
                    (White, King) => 0xE154,
                    (White, Knight) => 0xE158,
                    (White, Pawn) => 0xE159,
                    (White, Queen) => 0xE155,
                    (White, Rook) => 0xE156,
                }
            }
        },
        None => if white_square {
                    0xA0
                }
                else {
                    0xE100
                },
    }
}

fn square_to_maybe_indexes(square: &Square) -> (Option<usize>, Option<usize>) {
    let string = format!("{:?}", square).to_lowercase();
    let mut chars = string.chars();
    let column = chars.next().unwrap();
    let line = chars.next().unwrap();
    let x =
        match column {
            'a' => Some(0),
            'b' => Some(1),
            'c' => Some(2),
            'd' => Some(3),
            'e' => Some(4),
            'f' => Some(5),
            'g' => Some(6),
            'h' => Some(7),
            'x' => None,
            _ => unreachable!(),
        };
    let y =
        if line == 'x' {
            None
        }
        else {
            Some(8 - (line as usize - '0' as usize))

        };
    (x, y)
}

fn square_to_indexes(square: &Square) -> (usize, usize) {
    let string = format!("{:?}", square).to_lowercase();
    let mut chars = string.chars();
    let column = chars.next().unwrap();
    let line = chars.next().unwrap();
    let x =
        match column {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => unreachable!(),
        };
    let y = 8 - (line as usize - '0' as usize);
    (x, y)
}

fn coord_valid(x: i32) -> bool {
    x >= 0 && x < 8
}

fn is_valid(x: i32, y: i32) -> bool {
    coord_valid(x) && coord_valid(y)
}
