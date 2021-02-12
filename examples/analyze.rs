use quintet::encoding;
use quintet::{analyzer, board};
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
        let mut board = board::Board::new();
        for p in &blacks {
            board = board.put(true, p);
        }
        for p in &whites {
            board = board.put(false, p);
        }
        println!("\nBoard: \n{}", board.to_string());

        println!("Forbiddens:");
        for (p, kind) in analyzer::forbiddens(&board) {
            println!("\t{:?} {:?}", p, kind)
        }

        println!("Black twos:");
        for row in analyzer::rows(&board, true, analyzer::RowKind::Two) {
            println!("\t{:?}", row)
        }

        println!("Black swords:");
        for row in analyzer::rows(&board, true, analyzer::RowKind::Sword) {
            println!("\t{:?}", row)
        }

        println!("Black threes:");
        for row in analyzer::rows(&board, true, analyzer::RowKind::Three) {
            println!("\t{:?}", row)
        }

        println!("Black fours:");
        for row in analyzer::rows(&board, true, analyzer::RowKind::Four) {
            println!("\t{:?}", row)
        }

        println!("White twos:");
        for row in analyzer::rows(&board, false, analyzer::RowKind::Two) {
            println!("\t{:?}", row)
        }

        println!("White swords:");
        for row in analyzer::rows(&board, false, analyzer::RowKind::Sword) {
            println!("\t{:?}", row)
        }

        println!("White threes:");
        for row in analyzer::rows(&board, false, analyzer::RowKind::Three) {
            println!("\t{:?}", row)
        }

        println!("White fours:");
        for row in analyzer::rows(&board, false, analyzer::RowKind::Four) {
            println!("\t{:?}", row)
        }
    }
}
