use crate::evaluate::{evaluate_position, evaluate_outcome};
use crate::transposition::{TranspositionTable, Bound, TTEntry};
use shakmaty::{Chess, Move, Position, MoveList, zobrist::{Zobrist64,ZobristHash}, EnPassantMode};
use std::time::Instant;

pub fn negamax(
    board: &Chess, 
    depth: usize, 
    ply: usize, 
    end_time: Instant,
    pvs: bool,
    mut alpha: i32,
    beta: i32,
    t_table: &mut TranspositionTable,
    evaluate_count: &mut usize) -> (Option<Move>, i32) {

    // Check if the game is over (checkmate or stalemate)
    if let Some(outcome) = board.outcome() {
        *evaluate_count += 1;
        return (None, evaluate_outcome(outcome, board.turn(), &ply));
    } 

    // Check if we are at terminal depth
    if depth == 0 { 
        //*evaluate_count += 1;
        //return (None, evaluate_position(board));
        return (None, quiesce(board, 0, alpha, beta, evaluate_count));
    }

    // Generate a hash for the board
    let hash = board.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);

    // Lookup this hash in the transposition table
    if let Some(entry) = t_table.lookup(hash) {
        if entry.depth >= depth  { // If this position has been adequately explored before 
            match entry.bound {
                Bound::Exact => return (entry.best_move.clone(), entry.value), // Exact evaluation, reuse
                Bound::LowerBound => if entry.value >= beta { return (entry.best_move.clone(), entry.value); }, // This entry is at least as good as beta, cutoff
                Bound::UpperBound => if entry.value <= alpha { return (entry.best_move.clone(), entry.value); }, // This entry is worse than our current best, cutoff
            }        
        }
    }

    // Get a list of legal moves and initialize the best move & score
    let mut legal_moves: MoveList = board.legal_moves();
    let mut best_move: Option<Move> = None;
    let mut best_value = -i32::MAX;

    // Sort moves to get the best move first
    legal_moves = sort_moves(board, legal_moves, t_table);

    // Evaluate every legal move at one ply deeper
    for mve in legal_moves{ 
        let mut board_copy: Chess = board.clone(); // Copy the current state of the board
        board_copy.play_unchecked(&mve); // Play the move under consideration
        let (_, value): (Option<Move>, i32); // Initialize values to hold the evaluation results
        (_, value) = negamax(&board_copy, depth-1, ply+1, end_time, true, -beta, -alpha, t_table, evaluate_count);

        /* WIP Negascout implementation - currently this causes Elo loss
        if (i == 0) && (pvs) {
            (_, value) = negamax(&board_copy, depth-1, ply+1, end_time, true, -beta, -alpha, t_table, evaluate_count);
        } else {
            // Scout search with a null window
            (_, value) = negamax(&board_copy, depth-1, ply+1, end_time, false, -alpha-1, -alpha, t_table, evaluate_count);
            if (alpha < -value) && (-value < beta) {
                // If the search went outside the window then re-do it with a full window
                (_, value) = negamax(&board_copy, depth-1, ply+1, end_time, false, -beta, -alpha, t_table, evaluate_count);
            }
        }
        */

        if -value > best_value {
            best_value = -value;   // Track the best value
            best_move = Some(mve); // Track the best move
        }

        alpha = alpha.max(-value); // Update alpha
        if alpha >= beta { break } // Alpha-beta pruning

        // If time is expired stop searching and return what you have
        if Instant::now() >= end_time {return (best_move, best_value);}
    }

    // Determine the bound type to store in the TT
    let (bound, stored_value) = if best_value <= alpha { (Bound::UpperBound, alpha) }
    else if best_value >= beta { (Bound::LowerBound, beta) }
    else { (Bound::Exact, best_value) };

    // Store the result in the transposition table
    let entry = TTEntry {
        value: stored_value,
        best_move: best_move.clone(),
        depth: depth,
        bound: bound,
    };
    t_table.store(hash, entry);

    return (best_move, best_value);

}

fn quiesce(
    board: &Chess,
    ply: usize, 
    mut alpha: i32,
    beta: i32,
    evaluate_count: &mut usize) -> i32 {

    // Check if the game is over (checkmate or stalemate)
    if let Some(outcome) = board.outcome() {
        *evaluate_count += 1;
        return evaluate_outcome(outcome, board.turn(), &ply);
    } 

    // Take the static score of this node
    *evaluate_count += 1;
    let stand_pat_score = evaluate_position(&board);

    // If at terminal ply return the current static evaluation
    if ply >= 3 {
        return stand_pat_score;
    }

    if stand_pat_score >= beta {return beta;} // This position is too good, so fail high
    alpha = alpha.max(stand_pat_score); // This position is better than our current best, so update alpha

    // Get a list of legal moves and initialize the best move & score
    let legal_moves: MoveList = board.legal_moves();

    for mve in legal_moves{
        let value: i32;

        // If the move is quiet (not a capture or promotion), don't do anything
        if !mve.is_capture() && !mve.is_promotion(){continue;} 
        else {
            let mut board_copy: Chess = board.clone(); // Copy the current state of the board
            board_copy.play_unchecked(&mve); // Play the move under consideration
            value = quiesce(&board_copy, ply+1, -beta, -alpha, evaluate_count);
        }

        alpha = alpha.max(-value); // Update alpha
        if alpha >= beta {
            break // Alpha-beta pruning
        }
    }

    return alpha;
    
}

pub fn iterative_deepening(
    board: &Chess,
    max_depth: usize,
    end_time: Instant,
    t_table: &mut TranspositionTable,
    evaluate_count: &mut usize) -> (Option<Move>, i32, usize) {
        
    let mut best_move: Option<Move> = None;
    let mut best_eval = -i32::MAX;
    let mut max_depth_reached: usize = 0;

    for depth in 1..(max_depth+1) {
        let depth_start_time = Instant::now();
        /* WIP Aspiration Window code 
        let (mut alpha, mut beta): (i32, i32);
        let (mut mv, mut score): (Option<Move>, i32);
        if depth <= 3{ (alpha, beta) = (-i32::MAX, i32::MAX); } // No aspiration window at shallow depth
        else {(alpha, beta) = (best_eval-30, best_eval+30); } // 50 centipawn aspiration window
        loop {
            (mv, score) = negamax(board, depth, 0, end_time, true, alpha, beta, t_table, evaluate_count);
            if -score >= alpha && -score <= beta { break; }
            else if -score < alpha { alpha -= 200; println!("Failed low depth {}", depth);}
            else if -score > beta { beta += 200; println!("Failed high depth {}", depth);}
        }
        */
        
        let (mv, score) = negamax(board, depth, 0, end_time, true, -i32::MAX, i32::MAX, t_table, evaluate_count);
        
        let depth_duration = depth_start_time.elapsed();
        if Instant::now() >= end_time {break;}

        if let Some(mv) = mv {
            best_eval = score;
            best_move = Some(mv);
            max_depth_reached = depth;
        
        if (Instant::now() + 4*depth_duration) >= end_time {
            return (best_move, best_eval, max_depth_reached);
            }
        }
        //println!("{} - {} - {}", depth, evaluate_count, depth_duration.as_millis());
    }
    (best_move, best_eval, max_depth_reached)
}

// Sort moves based on priority.
fn sort_moves(
    board: &Chess,
    mut moves: MoveList,
     t_table: &mut TranspositionTable
    ) -> MoveList {

    // Check what we previously thought was the best move for this position
    let hash = board.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
    let tt_best_move : Option<Move>;
    if let Some(tt_entry) = t_table.lookup(hash) {
        tt_best_move = tt_entry.best_move;
    } else {
        tt_best_move = None;
    }

    // Sort moves by priority, where lower values mean higher priority.
    moves.sort_by_key(|mov| {
        // Get the principal variation
        // Check if the move is the best move from the transposition table.
        if Some(mov) == tt_best_move.as_ref() { return 1; }
        // Check if the move is a capture.
        if mov.is_capture() { return 2; }
        // Check if the move is a promotion.
        if mov.is_promotion() { return 3; }
        // Quiet moves have the lowest priority.
        4}
    );
    return moves;
}