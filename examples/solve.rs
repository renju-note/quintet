use quintet::bitboard::*;
use quintet::solver;
use std::io;
use std::time::Instant;

fn main() {
    loop {
        println!("\nBoard code (blacks/whites):");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let result = code.parse::<Board>();
        let mut board = match result {
            Ok(board) => board,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        println!("\nBoard:\n{}", board);

        println!("\nBlack VCF:");
        let start = Instant::now();
        let result = solver::solve(u8::MAX, &mut board, true);
        let elapsed = start.elapsed();
        println!("\tElapsed: {:?}", elapsed);
        match result {
            Some(ps) => {
                println!("\t{} times", (ps.len() + 1) / 2);
                println!("\t{}", points_to_string(&ps));
            }
            None => println!("\tNone"),
        }

        println!("\nWhite VCF:");
        let start = Instant::now();
        let result = solver::solve(u8::MAX, &mut board, false);
        let elapsed = start.elapsed();
        println!("\tElapsed: {:?}", elapsed);
        match result {
            Some(ps) => {
                println!("\t{} times", (ps.len() + 1) / 2);
                println!("\t{}", points_to_string(&ps));
            }
            None => println!("\tNone"),
        }
    }
}

fn points_to_string(ps: &[Point]) -> String {
    ps.iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
