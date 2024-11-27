mod uci;
mod parsers;
mod search;
mod evaluate;
mod transposition;

use transposition::TranspositionTable;
use uci::{go, is_ready, perft, print_info, set_position, quit};

use std::io::{self};
use shakmaty::Chess;

fn main() {

    // Initialize a new chess board (start from the standard initial position)
    let mut board = Chess::default();

    // Initialize a new transpositon table
    let mut t_table = TranspositionTable::new();

    // Start reading input from stdin
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        // Clear the input buffer
        input.clear();
        
        // Read a line of input from stdin
        stdin.read_line(&mut input).unwrap();
        let input: &str = input.trim(); // Remove leading/trailing whitespace

        if input == "uci" { print_info(); }
        else if input == "isready" { is_ready(); }
        else if input == "quit" { quit(); }
        else if input.starts_with("perft") { perft(input, &board); }
        else if input.starts_with("position") { board = set_position(input, board); }
        else if input.starts_with("go") { go(input, &board, &mut t_table); }

    }

}