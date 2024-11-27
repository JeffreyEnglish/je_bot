use crate::parsers::{parse_perft_command, parse_go_command};
use crate::search::iterative_deepening;
use crate::transposition::TranspositionTable;

use std::time::{Instant, Duration};
use std::process;

use shakmaty::{Chess, Position, uci::UciMove, CastlingMode};

pub fn print_info() {
    println!("id name je_bot");
    println!("id author Jeffrey English");
    println!("uciok");
}

pub fn is_ready() {
    println!("readyok");
}

pub fn perft(input: &str, board: &Chess) {
    let depth: usize = parse_perft_command(input);
    let start: Instant = Instant::now();
    let graph_size = perft_search(depth, &board);
    let time: u128 = start.elapsed().as_millis();
    println!("Perft depth {} nodes {} time {}ms ({}Mn/sec)", depth, graph_size, time, graph_size as f32 / time as f32 / 1000.);
}

pub fn set_position(input: &str, mut board: Chess) -> Chess {
    // UCI command could look like: position startpos moves e2e4 e7e5
    if input.contains("startpos") {
        board = Chess::default();  // Start from the default starting position
    }
    
    if let Some(moves) = input.split("moves").nth(1) {
        let moves: Vec<&str> = moves.trim().split_whitespace().collect();
        // Apply each move to the board
        for mov in moves {
            let uci: UciMove = mov.parse().unwrap();
            let m = uci.to_move(&board).unwrap();
            board.play_unchecked(&m);
            } 
    }
    return board.to_owned();
}

pub fn go(input: &str, board: &Chess, t_table: &mut TranspositionTable) {
    let (time, inc) = parse_go_command(input, board.turn().is_white());
    let end_time: Instant = Instant::now() + Duration::from_millis((time/20+inc).max(1000));
    let max_depth= 18;
    let mut evaluate_count = 0;
    //let start: Instant = Instant::now();
    //let (best_move, best_score) = negamax(board, depth, ply, -i32::MAX, i32::MAX, &mut evaluate_count);
    let (best_move, best_score, _max_depth) = iterative_deepening(board, max_depth, end_time, t_table, &mut evaluate_count);
    //let time: u128 = start.elapsed().as_millis();
    //println!("Searched {} nodes in {}ms ({}kn/sec)", evaluate_count, time, evaluate_count as f32 / time as f32);
    println!("info score cp {}", best_score);
    println!("bestmove {}", best_move.unwrap().to_uci(CastlingMode::Standard));
}

pub fn quit() { process::exit(0);}

fn perft_search(depth: usize, board: &Chess) -> usize {
    if depth == 0 {return 1}
    
    // Generate all legal moves from the current position
    let mut count = 0;
    let moves = board.legal_moves();
    for mv in moves {
        // Apply the move to get the new board state
        let mut next_position = board.clone();
        next_position.play_unchecked(&mv);
        // Recursive call with decreased depth
        count += perft_search(depth - 1, &next_position);
    }
        return count
}