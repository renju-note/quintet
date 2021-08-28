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
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        println!("\nBoard:\n{}", board);

        println!("\nRows:");
        for player in &[Player::Black, Player::White] {
            println!("  {:?}", player);
            for kind in &[
                RowKind::Two,
                RowKind::Three,
                RowKind::Sword,
                RowKind::Four,
                RowKind::Five,
            ] {
                println!("    {:?}:", kind);
                for row in board.rows(*player, *kind) {
                    println!("      {}", row)
                }
            }
        }

        println!("\nForbiddens:");
        for (kind, p) in board.forbiddens() {
            println!("  {:?}\t{}", kind, p)
        }
    }
}
