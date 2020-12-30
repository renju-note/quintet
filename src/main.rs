use std::io;
mod board;

fn main() {
    loop {
        println!("blacks: ");
        let mut blacks = String::new();
        io::stdin().read_line(&mut blacks).expect("fail");
        let blacks: board::row::Stones = match u32::from_str_radix(&blacks.trim(), 2) {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("whites: ");
        let mut whites = String::new();
        io::stdin().read_line(&mut whites).expect("fail");
        let whites: board::row::Stones = match u32::from_str_radix(&whites.trim(), 2) {
            Ok(num) => num,
            Err(_) => continue,
        };

        let mut line = match board::line::Line::new(15, blacks, whites) {
            Ok(line) => line,
            Err(_) => continue,
        };

        println!("Line: {}", line.to_string());

        println!("Two:");
        for lr in line.rows(true, board::row::RowKind::Two) {
            println!("    {}, {}, {:?}", lr.start, lr.size, lr.eyes)
        }

        println!("Sword:");
        for lr in line.rows(true, board::row::RowKind::Sword) {
            println!("    {}, {}, {:?}", lr.start, lr.size, lr.eyes)
        }

        println!("Three:");
        for lr in line.rows(true, board::row::RowKind::Three) {
            println!("    {}, {}, {:?}", lr.start, lr.size, lr.eyes)
        }

        println!("Four:");
        for lr in line.rows(true, board::row::RowKind::Four) {
            println!("    {}, {}, {:?}", lr.start, lr.size, lr.eyes)
        }

        println!("Five:");
        for lr in line.rows(true, board::row::RowKind::Five) {
            println!("    {}, {}, {:?}", lr.start, lr.size, lr.eyes)
        }

        println!("Overline:");
        for lr in line.rows(true, board::row::RowKind::Five) {
            println!("    {}, {}, {:?}", lr.start, lr.size, lr.eyes)
        }
    }
}
