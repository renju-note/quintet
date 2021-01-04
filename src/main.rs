mod analyzer;
mod board;
mod encoding;

use analyzer::RowKind;
use std::io;

fn main() {
    let coder = encoding::Coder::new();
    let mut analyzer = analyzer::Analyzer::new();

    loop {
        println!("Game code: ");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let points = match coder.decode(&code) {
            Ok(points) => points,
            Err(_) => continue,
        };

        let mut board = board::Board::new();
        let mut black = true;
        for p in points {
            board = board.put(black, p);
            black = !black
        }

        println!("\nBoard: \n{}", board.to_string());

        println!("Forbiddens:");
        for (p, kind) in analyzer.forbiddens(&board) {
            println!("    {:?} {:?}", p, kind)
        }

        println!("Black threes:");
        for row in analyzer.rows(&board, true, RowKind::Three) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }

        println!("Black fours:");
        for row in analyzer.rows(&board, true, RowKind::Four) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }

        println!("White threes:");
        for row in analyzer.rows(&board, false, RowKind::Three) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }

        println!("White fours:");
        for row in analyzer.rows(&board, false, RowKind::Four) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }
    }
}
