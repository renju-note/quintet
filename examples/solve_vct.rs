use quintet::board::*;
use quintet::mate;
use std::io;
use std::time::Instant;

fn main() {
    loop {
        println!("\nBoard code (blacks/whites):");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let result = code.parse::<Board>();
        let board = match result {
            Ok(board) => board,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        println!("\nBoard:\n{}", board);

        println!("\nBlack VCF:");
        let start = Instant::now();
        let result = mate::solve_vct(&board, Player::Black, 10, 5);
        let elapsed = start.elapsed();
        println!("\tElapsed: {:?}", elapsed);
        match result {
            Some(ps) => {
                println!("\t{} times", (ps.len() + 1) / 2);
                println!("\t{}", Points(ps));
            }
            None => println!("\tNone"),
        }

        println!("\nWhite VCF:");
        let start = Instant::now();
        let result = mate::solve_vct(&board, Player::White, 10, 5);
        let elapsed = start.elapsed();
        println!("\tElapsed: {:?}", elapsed);
        match result {
            Some(ps) => {
                println!("\t{} times", (ps.len() + 1) / 2);
                println!("\t{}", Points(ps));
            }
            None => println!("\tNone"),
        }
    }
}
