use std::collections::HashMap;

pub fn parse_perft_command(input: &str) -> usize {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.len() == 0 {
        return 0
    } else {
        return tokens[1].parse::<usize>().unwrap_or(1);
    }
}

pub fn parse_go_command(input: &str, is_white_turn: bool) -> (u64, u64) {
    // Default time and increment values
    let mut wtime: u64 = 0;
    let mut btime: u64 = 0;
    let mut winc: u64 = 0;
    let mut binc: u64 = 0;

    // Create a HashMap for easy parsing of arguments
    let mut options: HashMap<&str, u64> = HashMap::new();

    // Split the input and extract key-value pairs
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut i: usize = 1;  // Start after the "go" token
    while i < tokens.len() {
        if i + 1 < tokens.len() {
            if let Ok(value) = tokens[i + 1].parse::<u64>() {
                options.insert(tokens[i], value);
            }
        }
        i += 2;
    }

    // Extract values for wtime, btime, winc, binc
    if let Some(&val) = options.get("wtime") { wtime = val; }
    if let Some(&val) = options.get("btime") { btime = val; }
    if let Some(&val) = options.get("winc") { winc = val; }
    if let Some(&val) = options.get("binc") { binc = val; }

    // Return the correct time and increment for the current player
    if is_white_turn {
        (wtime, winc)
    } else {
        (btime, binc)
    }
}