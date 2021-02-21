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
        for p in &blacks {
            board.put(true, p);
        }
        for p in &whites {
            board.put(false, p);
        }
        println!("\nBoard: \n{}", board.to_string());

        println!("Forbiddens:");
        for (p, kind) in forbiddens(&board) {
            println!("\t{:?} {:?}", p, kind)
        }

        println!("Black twos:");
        for row in board.rows(true, RowKind::Two) {
            println!("\t{:?}", row)
        }

        println!("Black swords:");
        for row in board.rows(true, RowKind::Sword) {
            println!("\t{:?}", row)
        }

        println!("Black threes:");
        for row in board.rows(true, RowKind::Three) {
            println!("\t{:?}", row)
        }

        println!("Black fours:");
        for row in board.rows(true, RowKind::Four) {
            println!("\t{:?}", row)
        }

        println!("White twos:");
        for row in board.rows(false, RowKind::Two) {
            println!("\t{:?}", row)
        }

        println!("White swords:");
        for row in board.rows(false, RowKind::Sword) {
            println!("\t{:?}", row)
        }

        println!("White threes:");
        for row in board.rows(false, RowKind::Three) {
            println!("\t{:?}", row)
        }

        println!("White fours:");
        for row in board.rows(false, RowKind::Four) {
            println!("\t{:?}", row)
        }
    }
}
