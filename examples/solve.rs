use quintet::board::*;
use quintet::mate::*;
use std::env;
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let args = env::args().collect::<Vec<String>>();
    let kind = args[1].parse::<SolverKind>()?;
    println!("Kind: {:?}", kind);
    let max_depth = args[2].parse::<u8>().map_err(|_| "ParseIntError")?;
    println!("MaxDepth: {:?}", max_depth);
    let turn = args[3].parse::<Player>()?;
    println!("Player: {:?}", turn);
    let board = args[4].parse::<Board>()?;
    println!("Board:\n{}\n", board.to_pretty_string());
    solve_print(kind, max_depth, board, turn);
    Ok(())
}

fn solve_print(kind: SolverKind, max_depth: u8, board: Board, turn: Player) {
    println!("Solving...\n");
    let start = Instant::now();
    let solution = solve(kind, max_depth, &board, turn);
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
