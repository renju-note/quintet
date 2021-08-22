use quintet::encoding;
use quintet::solver;
use std::io;
use std::time::Instant;

fn main() {
    loop {
        println!("\nBoard code (blacks/whites):");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let mut board = match encoding::decode_board(&code) {
            Ok(board) => board,
            Err(s) => {
                println!("{}", s);
                continue;
            }
        };
        println!("\nBoard:\n{}", board.to_string());

        println!("\nBlack VCF:");
        let start = Instant::now();
        let result = solver::solve(u8::MAX, &mut board, true);
        let elapsed = start.elapsed();
        println!("\tElapsed: {:?}", elapsed);
        match result {
            Some(ps) => {
                println!("\t{} times", (ps.len() + 1) / 2);
                println!("\t{}", encoding::encode(&ps).unwrap());
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
                println!("\t{}", encoding::encode(&ps).unwrap());
            }
            None => println!("\tNone"),
        }
    }
}
