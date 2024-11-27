> je_bot is a UCI chess engine made to demonstrate important chess AI elements

The engine is written from scratch in Rust using [shakmaty](https://github.com/niklasf/shakmaty) for move generation and bitboard creation.

## Search 
The search uses a negamax approach with iterative deepening and alpha-beta pruning. Other enhancements:
- Transposition tables
- Quiescence search
- Move order (TT move > captures > promotions > other)

## Evaluation  
Evaluation is primarily through piece-square tables. A single table is used for all pieces except the king, which uses a seperate end-game table. Other heuristics:
- Penalty of stacked and isolated pawns
- Bonus for rooks on open files
- Bonus for keeping both bishops
- Bonus for king on a closed file

## Getting started
The code can be compiled using `cargo run --release`

## Lichess
The bot plays periodically on Lichess as https://lichess.org/@/je_bot. Running on an Fsv2 virtual machine it can evaluate roughly 1Mn/sec.
