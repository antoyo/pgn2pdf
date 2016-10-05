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

/*
 * TODO: improve error handling.
 * TODO: ask before overriding file.
 * TODO: open preview in another process.
 */

extern crate chess_pgn_parser;
extern crate docopt;
extern crate open;
extern crate rustc_serialize;
extern crate tempdir;

mod game;

use std::collections::HashMap;
use std::error::Error;
use std::ffi::{OsString};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::iter::{once, repeat};
use std::path::PathBuf;
use std::process::Command;

use chess_pgn_parser::{Game, GameMove, Square, read_games};
use chess_pgn_parser::AnnotationSymbol::{Blunder, Brilliant, Dubious, Good, Interesting, Mistake};
use chess_pgn_parser::Move::{BasicMove, CastleKingside, CastleQueenside};
use chess_pgn_parser::MoveNumber::{Black, White};
use chess_pgn_parser::Piece::{self, Bishop, King, Knight, Pawn, Queen, Rook};
use docopt::Docopt;
use tempdir::TempDir;

use game::ChessGame;
use self::ShowMoveOptions::*;

const MOVES_TO_SHOW: usize = 9;

#[derive(PartialEq)]
enum ShowMoveOptions {
    Normal,
    WithoutNum,
}

#[derive(Debug)]
struct ParseError {
    error: String,
}

impl ParseError {
    fn new(error: &str) -> Self {
        ParseError {
            error: error.to_string(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter) -> std::result::Result<(), fmt::Error> {
        write!(formatter, "{}", self.error)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.error
    }
}

type Result<T> = std::result::Result<T, Box<Error>>;

const THEME_DIR: &'static str = "/usr/local/share/pgn2pdf/";
const USAGE: &'static str = "
PGN to PDF converter.

Usage:
  pgn2pdf <filename> [--output=<output>]
  pgn2pdf <filename> [--preview]

Options:
  -o --output=<output>  Set output file.
  -p --preview          Preview the file in the system PDF viewer instead of saving it to a file.
  -h --help             Show this screen.
  --version             Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_filename: String,
    flag_output: Option<String>,
    flag_preview: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|decoder| decoder.decode())
        .unwrap_or_else(|error| error.exit());
    match convert(&args.arg_filename, args.flag_output, args.flag_preview) {
        Ok(output) =>
            if args.flag_preview {
                open_pdf_viewer(&output);
            },
        Err(error) => println!("{}", error),
    }
}

fn convert(filename: &str, output: Option<String>, preview: bool) -> Result<String> {
    let tempdir = try!(TempDir::new("pgn2pdf"));
    let games = try!(read_pgn_games(filename));
    let output = output.unwrap_or_else(|| {
        let mut output = PathBuf::from(&filename);
        output.set_extension("pdf");
        let output = output.file_name();
        let output = output.unwrap().to_str().unwrap().to_string();
        if preview {
            format!("/tmp/{}", output)
        }
        else {
            output
        }
    });
    // TODO: ask for which game to print.
    for game in games {
        let input = try!(write_asciidoc(&tempdir, game, filename));
        try!(run_asciidoc(input, &output));
    }
    Ok(output)
}

fn open_pdf_viewer(filename: &str) {
    if let Err(error) = open::that(filename) {
        println!("{}", error);
    }
}

fn run_asciidoc(filename: OsString, output: &str) -> Result<()> {
    let _ = try!(Command::new("asciidoctor-pdf")
        .arg(&filename)
        .arg("-o")
        .arg(output)
        .status());
    Ok(())
}

fn read_pgn_games(input: &str) -> Result<Vec<Game>> {
    let mut file = try!(File::open(input));
    let mut content = String::new();
    try!(file.read_to_string(&mut content));
    if let Ok(games) = read_games(&content) {
        Ok(games)
    }
    else {
        Err(Box::new(ParseError::new("parse error")))
    }
}

fn get_diagram(moves: &[&GameMove]) -> String {
    let mut game = ChessGame::initial();
    for game_move in moves {
        game.play(game_move);
    }
    game.show()
}

fn get_initial_moves(game: &Game) -> Vec<&GameMove> {
    game.moves.iter()
        .take_while(|game_move| game_move.variations.is_empty())
        .collect()
}

fn get_title(game: &Game) -> String {
    let tags: HashMap<String, String> = game.tags.iter().cloned().collect();
    match (tags.get("White"), tags.get("Black")) {
        (Some(opening), Some(variation)) => format!("{} - {}", opening, variation),
        (None, Some(name)) | (Some(name), None) => name.clone(),
        (None, None) => String::new(),
    }
}

fn is_black_move(game_move: &&GameMove) -> bool {
    if let Some(ref number) = game_move.number {
        if let Black(_) = *number {
            true
        }
        else {
            false
        }
    }
    else {
        true
    }
}

fn is_white_move(game_move: &&GameMove) -> bool {
    if let Some(ref number) = game_move.number {
        if let White(_) = *number {
            return true;
        }
    }
    false
}

fn extract_variations(moves: &[GameMove]) -> String {
    // FIXME: if first move is black, take one less white move.
    let white_moves: Vec<_> = moves.iter()
        .filter(is_white_move)
        .take(MOVES_TO_SHOW)
        .map(|game_move| move_to_string(game_move, WithoutNum))
        .collect();
    let black_moves: Vec<_> = moves.iter()
        .filter(is_black_move)
        .take(MOVES_TO_SHOW)
        .map(|game_move| move_to_string(game_move, WithoutNum))
        .collect();
    // TODO: add ¹ for comments and variations.
    // TODO: add variation evaluation.
    let remaining_white = MOVES_TO_SHOW - white_moves.len();
    let rest_of_white_row: Vec<_> = repeat("|").take(remaining_white).collect();
    let remaining_black = MOVES_TO_SHOW - black_moves.len();
    let rest_of_black_row: Vec<_> = repeat("|").take(remaining_black).collect();
    format!("| {}\n{}\n| | {}\n{}\n", white_moves.join("\n| "), rest_of_white_row.join("\n"),
        black_moves.join("\n| "), rest_of_black_row.join("\n"))
}

fn get_variations(game: &Game, start_move_num: usize) -> String {
    let mut result = String::new();
    let mut moves = game.moves.iter()
        .skip(start_move_num);
    if let Some(start_move) = moves.next() {
        if !start_move.variations.is_empty() {
            result += &format!("[cols=\"1, {}*3\"]\n|===\n| ", MOVES_TO_SHOW);
            for num in start_move_num .. start_move_num + MOVES_TO_SHOW {
                result += &format!("|{} ", num);
            }
            result += "\n\n";
            let mut variations = vec![];
            variations.push(format!("| *1*\n{}", extract_variations(&game.moves[start_move_num..])));
            let separator = repeat("|")
                .take(MOVES_TO_SHOW)
                .chain(once("|\n"))
                .collect::<String>();
            for (index, variation) in start_move.variations.iter().enumerate() {
                variations.push(format!("| *{}*\n{}", index + 2, extract_variations(&variation.moves)));
            }
            result += &variations.join(&separator);
            result += "|===";
        }
    }
    result
}

fn write_asciidoc(tempdir: &TempDir, game: Game, filename: &str) -> Result<OsString> {
    let title = get_title(&game);
    let mut output = PathBuf::from(filename);
    output.set_extension("adoc");
    let filename = output.file_name();
    let output_file = filename.as_ref().unwrap();
    let file_path = tempdir.path().join(output_file);
    let mut file = try!(File::create(&file_path));
    //println!("{:#?}", game);
    let initial_moves = get_initial_moves(&game);
    let diagram = get_diagram(&initial_moves);
    let moves: Vec<String> = initial_moves.iter()
        .map(|game_move| move_to_string(game_move, Normal))
        .collect();
    let moves = moves.join(" ");
    let variations = get_variations(&game, initial_moves.len());
    let comments = "";
    //[cols="1,7,1,7"]
    //|===
    //|===
    try!(write!(file, include_str!("../themes/template.adoc"), THEME_DIR, title, diagram, moves, variations, comments));
    Ok(file_path.into_os_string())
}

fn move_to_string(game_move: &GameMove, options: ShowMoveOptions) -> String {
    let mut string = String::new();
    if options != WithoutNum {
        if let Some(White(number)) = game_move.number {
            string += &format!("{}.", number);
        }
    }
    let mov =
        match game_move.move_.move_ {
            BasicMove { ref from, is_capture, ref piece, ref promoted_to, ref to } => {
                let piece = piece_to_string(piece);
                let from = square_to_string(from);
                let symbol =
                    if is_capture {
                        "x"
                    }
                    else {
                        ""
                    };
                let to = square_to_string(to);
                let promotion =
                    if let Some(ref piece) = *promoted_to {
                        format!("={}", piece_to_string(piece))
                    }
                    else {
                        String::new()
                    };
                format!("{}{}{}{}{}", piece, from, symbol, to, promotion)
            },
            CastleKingside => "O-O".to_string(),
            CastleQueenside => "O-O-O".to_string(),
        };
    string += &mov;
    // TODO: support nags.
    // TODO: add ¹ for comments.
    if let Some(ref annotation) = game_move.move_.annotation_symbol {
        let annotation =
            match *annotation {
                Blunder => "??",
                Brilliant => "!!",
                Dubious => "?!",
                Good => "!",
                Interesting => "!?",
                Mistake => "?",
            };
        string += annotation;
    }
    if game_move.move_.is_check {
        string += "+";
    }
    else if game_move.move_.is_checkmate {
        string += "#";
    }
    string
}

fn piece_to_string(piece: &Piece) -> &str {
    // TODO: translate in english.
    match *piece {
        Bishop => "F",
        King => "R",
        Knight => "C",
        Pawn => "",
        Queen => "D",
        Rook => "T",
    }
}

fn square_to_string(square: &Square) -> String {
    format!("{:?}", square).to_lowercase().replace('x', "")
}
