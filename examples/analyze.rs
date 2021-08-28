use quintet::bitboard::*;
use std::io;

fn main() {
    loop {
        println!("\nBoard code (blacks/whites):");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let result = code.parse::<Board>();
        let mut board = match result {
            Ok(board) => board,
            Err(_) => {
                println!("ParseError");
                continue;
            }
        };
        println!("\nBoard:\n{}", board);

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
        for (p, kind) in board.forbiddens() {
            println!("  {:?}\t{:?}", kind, p)
        }
    }
}
