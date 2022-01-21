use quintet::board::*;
use quintet::mate;
use std::env;
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let args = env::args().collect::<Vec<String>>();
    let player = args[1].parse::<Player>()?;
    println!("Player: {:?}\n", player);
    let board = args[2].parse::<Board>()?;
    println!("Board:\n\n{}\n", board.to_pretty_string());

    solve(player, board);

    Ok(())
}

fn solve(player: Player, board: Board) {
    println!("Solving...\n");
    let start = Instant::now();
    let may_solution = mate::solve_vcf(&board, player, u8::MAX, true);
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
