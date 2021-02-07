use quintet::encoding;
use quintet::{Analyzer, Board, RowKind, VCFSolver};
use std::io;
use std::time::Instant;

fn main() {
    let mut analyzer = Analyzer::new(true, false);
    let mut solver = VCFSolver::new(true, false);

    loop {
        println!("Board code:");
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

        println!("Black VCF:");
        let vcf_start = Instant::now();
        let result = solver.solve(&board, true, u8::MAX, false);
        let vcf_duration = vcf_start.elapsed();
        match result {
            Some(ps) => println!("{}, {}", (ps.len() + 1) / 2, encoding::encode(&ps).unwrap()),
            None => println!("None"),
        }
        println!("Elapsed: {:?}", vcf_duration);

        println!("White VCF:");
        let vcf_start = Instant::now();
        let result = solver.solve(&board, false, u8::MAX, false);
        let vcf_duration = vcf_start.elapsed();
        match result {
            Some(ps) => println!("{}, {}", (ps.len() + 1) / 2, encoding::encode(&ps).unwrap()),
            None => println!("None"),
        }
        println!("Elapsed: {:?}", vcf_duration);
    }
}
