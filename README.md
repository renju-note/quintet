# quintet

Renju mate solver compilable to WebAssembly

## Run

### VCF

```
% cargo run --release --example solve vcf 10 5 x \
  H8,H7,F6,G7,I7,G9,G6,H6,F8,E8,F9,F7,E7,D6,H4,G5,J6,I5,J4,K5,J5,J3,J8,J7,I4,K4,K6,L7,L6,M6,I2,N7,L5,K7,M7,G4,I8,G8,K8,L8,D5,E4,F4,F3,H3,G2,E2,I10,J11,F11,G11,D9,E10,E12,F13,C10,B11,C12,D12
Mode: VCFDFS
Limit: 10
ThreatLimit: 5
Attacker: White
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

Elapsed: 38.75µs
End: Fours(B15, G10)
Times (Length): 8 (15)
Moves: G1,G3,E6,H9,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13
```

### VCT

```
% cargo run --release --example solve vct_dfpns 255 3 x \
  H8,I9,H7,J8,F8,H9,G9,H10,G11,F10,I10,G10,I8,J9,E10,K9,L9,G8,J10,K7,L6,K8,K6,L10,I7,J6,M9,I5,H4,F7,G6,E6,D5,F5,G4
Mode: VCTDFPNS
Limit: 255
ThreatLimit: 3
Attacker: White
Board:
15 . . . . . . . . . . . . . . .
14 . . . . . . . . . . . . . . .
13 . . . . . . . . . . . . . . .
12 . . . . . . . . . . . . . . .
11 . . . . . . o . . . . . . . .
10 . . . . o x x x o o . x . . .
 9 . . . . . . o x x x x o o . .
 8 . . . . . o x o o x x . . . .
 7 . . . . . x . o o . x . . . .
 6 . . . . x . o . . x o o . . .
 5 . . . o . x . . x . . . . . .
 4 . . . . . . o o . . . . . . .
 3 . . . . . . . . . . . . . . .
 2 . . . . . . . . . . . . . . .
 1 . . . . . . . . . . . . . . .
   A B C D E F G H I J K L M N O

Solving...

Elapsed: 903.095959ms
End: Forbidden(L7)
Times (Length): 15 (29)
Moves: K11,K10,N12,M11,N8,H5,H6,L8,J5,J7,M5,L4,M6,K5,J4,K3,J3,J2,K4,L5,M4,L3,L2,F3,I6,E2,D1,M3,N5
```

Original game: https://www.renju.net/media/games.php?gameid=92337

## Release

```
# build
$ wasm-pack build --scope renju-note

# publish
$ wasm-pack publish
```

See also: https://developer.mozilla.org/ja/docs/WebAssembly/Rust_to_wasm
