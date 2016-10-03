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

use chess_pgn_parser::GameMove;
use chess_pgn_parser::Piece::{self, Bishop, King, Knight, Pawn, Queen, Rook};

use self::Color::{Black, White};

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

    pub fn play(&self, game_move: &GameMove) {
    }

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
