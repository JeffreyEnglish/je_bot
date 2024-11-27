use shakmaty::{Chess, Position, Outcome, Color, Square, Piece, Role, Bitboard, File, Rank};

pub fn evaluate_outcome(outcome: Outcome, turn: Color, ply: &usize) -> i32 {
    match outcome {
        Outcome::Decisive { winner } => {
            if winner == turn {
                10_000 - *ply as i32 // The current player wins
            } else {
                -10_000 + *ply as i32 // The opponent wins
            }
        }
        // TODO - Shakmaty does not currently detect three-fold repition, should be tracked seperately
        Outcome::Draw => -50,  // Slight penalty for stalemate or draw
    }
}

pub fn evaluate_position(board: &Chess) -> i32 {
    // Always returns the score from the perspective of the player to play (White by convention)
    let mut white_material: i32 = 0;
    let mut black_material: i32 = 0;

    // Check if it's the endgame - if there are 4 of few top-level pieces left
    let endgame: bool = (board.board().sliders() | board.board().knights()).count() <= 4;
    
    // Iterate over all squares and accumulate scores
    for sq in Square::ALL {
        if let Some(piece) = board.board().piece_at(sq) {
    
            let material_value = piece_value(piece);
            let square_value = piece_square_value(piece, sq, endgame);
    
            if piece.color == board.turn() {
                white_material += material_value + square_value;
                if piece.role == Role::Pawn {white_material += passed_pawn_value(board, &sq, &piece.color);}
            } else {
                black_material += material_value + square_value;
                if piece.role == Role::Pawn {black_material += passed_pawn_value(board, &sq, &piece.color);}
            }
        }
    }
    let material_advantage = white_material - black_material;

    // Additional terms
    let sliding_open_file_advantage = open_file_value(board);
    let isolated_pawn_advantage = isolated_pawn_value(board);
    let king_protection_advantage = king_protection_value(board);
    let bishop_pair_advantage = bishop_pair_value(board);
    let stacked_pawn_advantage = stacked_pawn_value(board);
    return material_advantage + 
        sliding_open_file_advantage + 
        isolated_pawn_advantage + 
        king_protection_advantage + 
        bishop_pair_advantage + 
        stacked_pawn_advantage;
}

const PAWN_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
-40,-20,  0,  0,  0,  0,-20,-40,
-30,  0, 10, 15, 15, 10,  0,-30,
-30,  5, 15, 20, 20, 15,  5,-30,
-30,  0, 15, 20, 20, 15,  0,-30,
-30,  5, 10, 15, 15, 10,  5,-30,
-40,-20,  0,  5,  5,  0,-20,-40,
-50,-40,-30,-30,-30,-30,-40,-50
];

const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5, 10, 10,  5,  0,-10,
-10,  5,  5, 10, 10,  5,  5,-10,
-10,  0, 10, 10, 10, 10,  0,-10,
-10, 10, 10, 10, 10, 10, 10,-10,
-10,  5,  0,  0,  0,  0,  5,-10,
-20,-10,-10,-10,-10,-10,-10,-20
];

const ROOK_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

const QUEEN_TABLE: [i32;64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5,  5,  5,  5,  0,-10,
 -5,  0,  5,  5,  5,  5,  0, -5,
  0,  0,  5,  5,  5,  5,  0, -5,
-10,  5,  5,  5,  5,  5,  0,-10,
-10,  0,  5,  0,  0,  0,  0,-10,
-20,-10,-10, -5, -5,-10,-10,-20
];

const KING_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-20,-30,-30,-40,-40,-30,-30,-20,
-10,-20,-20,-20,-20,-20,-20,-10,
 20, 20,  0,  0,  0,  0, 20, 20,
 20, 30, 10,  0,  0, 10, 30, 20
];

const KING_TABLE_ENDGAME: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
-30,-20,-10,  0,  0,-10,-20,-30,
-30,-10, 20, 30, 30, 20,-10,-30,
-30,-10, 30, 40, 40, 30,-10,-30,
-30,-10, 30, 40, 40, 30,-10,-30,
-30,-10, 20, 30, 30, 20,-10,-30,
-30,-30,  0,  0,  0,  0,-30,-30,
-50,-30,-30,-30,-30,-30,-30,-50
];

/*
const PAWN_TABLE: [i32; 64] = [
    0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,];

const KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,];

const BISHOP_TABLE: [i32; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,];

const ROOK_TABLE: [i32; 64] = [
    32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,];

const QUEEN_TABLE: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,];

const KING_TABLE: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
    29,  -1, -20,  -7,  -8,  -4, -38, -29,
    -9,  24,   2, -16, -20,   6,  22, -22,
   -17, -20, -12, -27, -30, -25, -14, -36,
   -49,  -1, -27, -39, -46, -44, -33, -51,
   -14, -14, -22, -46, -44, -30, -15, -27,
     1,   7,  -8, -64, -43, -16,   9,   8,
   -15,  36,  12, -54,   8, -28,  24,  14,];

*/

fn piece_square_value(piece: Piece, square: Square, endgame: bool) -> i32 {

   let index: usize = if piece.color == Color::Black {
       square as usize // Use the square index directly for black
   } else {
       63 - (square as usize) // Mirror the index for white
   };

   match piece.role {
       Role::Pawn => PAWN_TABLE[index],
       Role::Knight => KNIGHT_TABLE[index],
       Role::Bishop => BISHOP_TABLE[index], 
       Role::Rook => ROOK_TABLE[index], 
       Role::Queen => QUEEN_TABLE[index],  
       Role::King => {
        if endgame {KING_TABLE_ENDGAME[index]}
        else {KING_TABLE[index]}
       }
   }
}

fn piece_value(piece: Piece) -> i32 {
    match piece.role {
        Role::Pawn => 100,
        Role::Knight => 300,
        Role::Bishop => 300,
        Role::Rook => 500,
        Role::Queen => 900,
        Role::King => 0, // King has no material value
    }
}

fn open_file_value(chess: &Chess) -> i32 {
    let to_play: Color = chess.turn();
    let board: &shakmaty::Board = chess.board();

    let player_bitboard: Bitboard = if to_play == Color::Black {board.black()} else {board.white()};
    let opponent_bitboard: Bitboard = if to_play == Color::White {board.black()} else {board.white()};

    let player_pawns = board.pawns() & player_bitboard;
    let opponent_pawns = board.pawns() & opponent_bitboard;
    let player_rq = board.rooks() & player_bitboard;
    let opponent_rq = board.rooks() & player_bitboard;
    let (mut player_score, mut opponent_score) = (0, 0);
    
    for file in File::ALL{
        let file_mask = Bitboard::from_file(file);
        // Open files
        if (file_mask & (player_pawns | opponent_pawns)).is_empty(){
            if (file_mask & player_rq).any() { player_score += 30; }
            if (file_mask & opponent_rq).any() { opponent_score += 30; }
        }
        // Semi-open files for player with a sliding piece
        else if (file_mask & player_pawns & !player_rq).is_empty() { player_score += 20; }
        // Semi-open files for opponent with a sliding piece
        else if (file_mask & opponent_pawns & !opponent_rq).is_empty() { opponent_score += 20; }
    }
    return player_score - opponent_score;
}

fn isolated_pawn_value(chess: &Chess) -> i32 {
    let mut player_score = 0;
    let board: &shakmaty::Board = chess.board();
    let pawnboard = board.pawns();

    // Player and opponent bitboard
    let to_play: Color = chess.turn();
    let player_bitboard: Bitboard = if to_play == Color::Black {board.black()} else {board.white()};
    let opponent_bitboard: Bitboard = if to_play == Color::White {board.black()} else {board.white()};

    for file in File::ALL{
        let file_mask = Bitboard::from_file(file);
        let neighbour_files = get_neighbour_files(file);

        // If player pawn is isolated
        if (neighbour_files & player_bitboard & pawnboard).is_empty() & (file_mask & player_bitboard & pawnboard).any() { player_score -= 10; }
        if (neighbour_files & opponent_bitboard & pawnboard).is_empty() & (file_mask & opponent_bitboard & pawnboard).any() { player_score += 10; }
    }
    return player_score
}

fn king_protection_value(chess: &Chess) -> i32 {
    let mut player_score = 0;
    let board: &shakmaty::Board = chess.board();
    let pawnboard = board.pawns();

    // Player and opponent bitboard
    let to_play: Color = chess.turn();
    let player_bitboard: Bitboard = if to_play == Color::Black {board.black()} else {board.white()};
    let opponent_bitboard: Bitboard = if to_play == Color::White {board.black()} else {board.white()};

    for file in File::ALL{
        let file_mask = Bitboard::from_file(file);
        if (file_mask & player_bitboard & board.kings()).is_empty() { continue; }
               
        // Penalty if the king is on an open file
        if (file_mask & player_bitboard & pawnboard).is_empty() { player_score -= 50; }
        if (file_mask & opponent_bitboard & pawnboard).is_empty() { player_score += 50; }

    }
    return player_score
}

fn bishop_pair_value(chess: &Chess) -> i32 {
    let mut player_score = 0;
    let board: &shakmaty::Board = chess.board();
    let bishopboard = board.bishops();
    let to_play: Color = chess.turn();
    let player_bitboard: Bitboard = if to_play == Color::Black {board.black()} else {board.white()};
    let opponent_bitboard: Bitboard = if to_play == Color::White {board.black()} else {board.white()};

    if (bishopboard & player_bitboard).count() == 2 { player_score += 30; }
    if (bishopboard & opponent_bitboard).count() == 1 { player_score -= 30; }
    
    return player_score;
}

fn stacked_pawn_value(chess: &Chess) -> i32 {
    let mut player_score = 0;
    let board: &shakmaty::Board = chess.board();
    let pawnboard = board.pawns();
    let to_play: Color = chess.turn();
    let player_bitboard: Bitboard = if to_play == Color::Black {board.black()} else {board.white()};
    let opponent_bitboard: Bitboard = if to_play == Color::White {board.black()} else {board.white()};
    for file in File::ALL{
        let file_mask = Bitboard::from_file(file);
               
        // Penalty if the king is on an open file
        if (file_mask & player_bitboard & pawnboard).count() > 1 { player_score -= 15; }
        if (file_mask & opponent_bitboard & pawnboard).count() > 1 { player_score += 15; }
    }
    return player_score
}

fn get_neighbour_files(file: File) -> Bitboard {
    let neighbour_files: Bitboard;
    
    if file == File::A {neighbour_files = Bitboard::from_file(File::B);}
    else if file == File::H {neighbour_files = Bitboard::from_file(File::G);}
    else {
        let file_mask = Bitboard::from_file(file);
        neighbour_files = file_mask.shift(-1) | file_mask.shift(1);
    }
    return neighbour_files
}


fn passed_pawn_value(chess: &Chess, square: &Square, color: &Color) -> i32 {
    let (r, f) = (square.rank(), square.file());
    let board: &shakmaty::Board = chess.board();

    let mut blocking_ranks = Bitboard::EMPTY;
    if color == &Color::White {
        for ahead in (r as u32+1)..8 {
            blocking_ranks |= Bitboard::from_rank(Rank::new(ahead));
        }
    } else {
        for ahead in 0..(r as u32) {
            blocking_ranks |= Bitboard::from_rank(Rank::new(ahead));
        }
    }

    let neighbour_files = get_neighbour_files(f);
    let pawnboard = board.pawns();
    let opponent_bitboard: Bitboard = if color == &Color::White {board.black()} else {board.white()};
    let file_mask = Bitboard::from_file(f);

    if ((file_mask | neighbour_files) & pawnboard & opponent_bitboard & blocking_ranks).is_empty() {return 20;}

    return 0;
}
