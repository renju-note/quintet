mod analyzer;
mod board;
mod encoding;
mod solver;

use analyzer::RowKind;
use std::io;
use std::time::Instant;

fn main() {
    let mut analyzer = analyzer::Analyzer::new();
    let mut solver = solver::VCFSolver::new();

    loop {
        println!("Game code: ");
        let mut code = String::new();
        io::stdin().read_line(&mut code).expect("fail");
        let points = match encoding::decode(&code.trim()) {
            Ok(points) => points,
            Err(_) => continue,
        };

        let mut board = board::Board::new();
        let mut black = true;
        for p in &points {
            board = board.put(black, p);
            black = !black
        }

        println!("\nBoard: \n{}", board.to_string());

        println!("Forbiddens:");
        for (p, kind) in analyzer.forbiddens(&board) {
            println!("    {:?} {:?}", p, kind)
        }

        println!("Black swords:");
        for row in analyzer.rows(&board, true, RowKind::Sword) {
            println!(
                "    {:?}, {:?}, {:?}, {:?}",
                row.direction, row.start, row.end, row.eyes
            )
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

        println!("White swords:");
        for row in analyzer.rows(&board, false, RowKind::Sword) {
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

        println!("VCF:");
        let vcf_start = Instant::now();
        let result = solver.solve(&board, black, u8::MAX, false);
        let vcf_duration = vcf_start.elapsed();
        match result {
            Some(ps) => println!("{}, {}", (ps.len() + 1) / 2, encoding::encode(&ps).unwrap()),
            None => println!("None"),
        }
        println!("Elapsed: {:?}", vcf_duration);

        println!("VCF(shortest):");
        let vcf_shortest_start = Instant::now();
        let result = solver.solve(&board, black, u8::MAX, true);
        let vcf_shortest_duration = vcf_shortest_start.elapsed();
        match result {
            Some(ps) => println!("{}, {}", (ps.len() + 1) / 2, encoding::encode(&ps).unwrap()),
            None => println!("None"),
        }
        println!("Elapsed: {:?}", vcf_shortest_duration);
    }
}
