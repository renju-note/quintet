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
        let result =
            board::row::search_pattern(blacks, whites, 15, true, board::row::RowKind::Three);
        println!("{:?}", result);
    }
}
