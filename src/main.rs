use std::io;
mod analyzer;
mod board;
mod encoding;
mod foundation;

use analyzer::row::RowKind;

fn main() {
    let coder = encoding::Coder::new();
    let mut analyzer = analyzer::analyzer::Analyzer::new();

    loop {
        println!("Game code: ");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let points = match coder.decode(&code) {
            Ok(points) => points,
            Err(_) => continue,
        };

        let mut square = match board::square::Square::new(foundation::N) {
            Ok(square) => square,
            Err(_) => continue,
        };
        let mut black = true;
        for p in points {
            square = square.put(black, p);
            black = !black
        }

        println!("\nSquare: \n{}", square.to_string());

        println!("Forbiddens:");
        for (p, kind) in analyzer.get_forbiddens(&square) {
            println!("    {:?} {:?}", p, kind)
        }

        println!("Black threes:");
        for row in analyzer.get_rows(&square, true, RowKind::Three) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }

        println!("Black fours:");
        for row in analyzer.get_rows(&square, true, RowKind::Four) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }

        println!("White threes:");
        for row in analyzer.get_rows(&square, false, RowKind::Three) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }

        println!("White fours:");
        for row in analyzer.get_rows(&square, false, RowKind::Four) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
        }
    }
}
