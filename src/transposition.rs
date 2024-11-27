use shakmaty::{zobrist::Zobrist64, Move};
use rustc_hash::FxHashMap;

type HashKey = Zobrist64; // A hash key representing the board position

pub struct TranspositionTable {
    pub table: FxHashMap<HashKey, TTEntry>,
}
#[derive(Copy, Clone)]
pub enum Bound {
    Exact,     // Exact i32
    LowerBound, // i32 is a lower bound
    UpperBound, // i32 is an upper bound
}

#[derive(Clone)]
pub struct TTEntry {
    pub value: i32,  // Evaluation value
    pub best_move: Option<Move>, // Best move found at this position
    pub depth: usize,   // Search depth at which this entry was created
    pub bound: Bound,   // Type of bound (α, β, or exact)
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: FxHashMap::default(),
        }
    }

    pub fn store(&mut self, hash: HashKey, entry: TTEntry) {
        let existing_entry = self.table.get(&hash).unwrap_or(&TTEntry{value: 0, best_move:None, depth: 0, bound: Bound::Exact});
        if entry.depth > existing_entry.depth {
            self.table.insert(hash, entry);
        }
    }

    pub fn lookup(&self, hash: HashKey) -> Option<&TTEntry> {
        self.table.get(&hash)
    }
}