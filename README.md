# quintet

Renju mate solver compilable to WebAssemby

## Run

```
$ cargo build --release --example solve_vcf

$ ./target/release/examples/solve_vcf x H8,H7,F6,G7,I7,G9,G6,H6,F8,E8,F9,F7,E7,D6,H4,G5,J6,I5,J4,K5,J5,J3,J8,J7,I4,K4,K6,L7,L6,M6,I2,N7,L5,K7,M7,G4,I8,G8,K8,L8,D5,E4,F4,F3,H3,G2,
E2,I10,J11,F11,G11,D9,E10,E12,F13,C10,B11,C12,D12
Player: White

Board:

15 . . . . . . . . . . . . . . .
14 . . . . . . . . . . . . . . .
13 . . . . . o . . . . . . . . .
12 . . x o x . . . . . . . . . .
11 . o . . . x o . . o . . . . .
10 . . x . o . . . x . . . . . .
 9 . . . x . o x . . . . . . . .
 8 . . . . x o x o o o o x . . .
 7 . . . . o x x x o x x x o x .
 6 . . . x . o o x . o o o x . .
 5 . . . o . . x . x o x o . . .
 4 . . . . x o x o o o x . . . .
 3 . . . . . x . o . x . . . . .
 2 . . . . o . x . o . . . . . .
 1 . . . . . . . . . . . . . . .
   A B C D E F G H I J K L M N O

Solving...

Elapsed: 1.414262ms
7 times
H5,I6,E6,H9,F5,E5,C8,D7,C11,C9,C14,C13,D13
```

## Release

```
# build
$ wasm-pack build --scope renju-note

# publish
$ wasm-pack publish
```

See also: https://developer.mozilla.org/ja/docs/WebAssembly/Rust_to_wasm
