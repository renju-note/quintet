use quintet::board::*;
use quintet::mate;
use std::env;
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let args = env::args().collect::<Vec<String>>();
    let turn = args[1].parse::<Player>()?;
    println!("Player: {:?}\n", turn);
    let board = args[2].parse::<Board>()?;
    println!("Board:\n\n{}\n", board.to_pretty_string());
    solve(board, turn);
    Ok(())
}

fn solve(board: Board, turn: Player) {
    println!("Solving...\n");
    let start = Instant::now();
    let may_solution = mate::solve_vct_pn(&board, turn, u8::MAX);
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    match may_solution {
        Some(solution) => {
            println!("{} times", (solution.len() + 1) / 2);
            println!("{}", Points(solution));
        }
        None => println!("None"),
    }
}
