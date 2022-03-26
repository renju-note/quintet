use quintet::board::*;
use quintet::mate::*;
use std::env;
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let args = env::args().collect::<Vec<String>>();

    let kind = args[1].parse::<SolveMode>()?;
    println!("Kind: {:?}", kind);

    let max_depth = args[2].parse::<u8>().map_err(|_| "ParseIntError")?;
    println!("MaxDepth: {:?}", max_depth);

    let threat_limit = args[3].parse::<u8>().map_err(|_| "ParseIntError")?;
    println!("ThreatLimit: {:?}", threat_limit);

    let turn = args[4].parse::<Player>()?;
    println!("Player: {:?}", turn);

    let board = args[5].parse::<Board>()?;
    println!("Board:\n{}\n", board.to_pretty_string());

    solve_print(kind, max_depth, board, turn, threat_limit);

    Ok(())
}

fn solve_print(kind: SolveMode, max_depth: u8, board: Board, turn: Player, threat_limit: u8) {
    println!("Solving...\n");
    let start = Instant::now();
    let solution = solve(kind, max_depth, &board, turn, threat_limit);
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    match solution {
        Some(m) => {
            println!("End: {}", m.end);
            println!("Times (Length): {} ({})", m.n_times(), m.n_moves());
            println!("Moves: {}", Points(m.path));
        }
        None => println!("None"),
    }
}
