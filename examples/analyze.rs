use quintet::board::{forbiddens, Board, RowKind};
use quintet::encoding;
use std::io;

fn main() {
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
        for p in blacks {
            board.put(true, p);
        }
        for p in whites {
            board.put(false, p);
        }
        println!("\nBoard:\n{}", board.to_string());

        println!("\nRows:");
        for black in &[true, false] {
            println!("  {}", if *black { "Black:" } else { "White:" });
            for kind in &[
                RowKind::Two,
                RowKind::Three,
                RowKind::Sword,
                RowKind::Four,
                RowKind::Five,
            ] {
                println!("    {:?}:", kind);
                for row in board.rows(*black, *kind) {
                    println!("      {:?}", row)
                }
            }
        }

        println!("\nForbiddens:");
        for (p, kind) in forbiddens(&board) {
            println!("  {:?}\t{:?}", kind, p)
        }
    }
}
