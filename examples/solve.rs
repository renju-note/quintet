use quintet::encoding;
use quintet::{Board, VCFSolver};
use std::io;
use std::time::Instant;

fn main() {
    let mut solver = VCFSolver::new();
    loop {
        println!("\nBoard code (blacks/whites):");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let codes = code.trim().split('/').collect::<Vec<_>>();
        if codes.len() != 2 {
            continue;
        }
        let blacks = match encoding::decode(codes[0]) {
            Ok(points) => points,
            Err(_) => continue,
        };
        let whites = match encoding::decode(codes[1]) {
            Ok(points) => points,
            Err(_) => continue,
        };
        let mut board = Board::new();
        for p in &blacks {
            board = board.put(true, p);
        }
        for p in &whites {
            board = board.put(false, p);
        }
        println!("\nBoard: \n{}", board.to_string());

        println!("Black VCF:");
        let start = Instant::now();
        let result = solver.solve(&board, true, u8::MAX);
        let elapsed = start.elapsed();
        match result {
            Some(ps) => {
                println!("\t{} moves", (ps.len() + 1) / 2);
                println!("\t{}", encoding::encode(&ps).unwrap());
            }
            None => println!("\tNone"),
        }
        println!("\tElapsed: {:?}\n", elapsed);

        println!("White VCF:");
        let start = Instant::now();
        let result = solver.solve(&board, false, u8::MAX);
        let elapsed = start.elapsed();
        match result {
            Some(ps) => {
                println!("\t{} moves", (ps.len() + 1) / 2);
                println!("\t{}", encoding::encode(&ps).unwrap());
            }
            None => println!("\tNone"),
        }
        println!("\tElapsed: {:?}\n", elapsed);
    }
}
