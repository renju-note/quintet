# quintet

Renju mate solver compilable to WebAssemby

## Run

```
$ cargo run --release --example solve
    Finished release [optimized] target(s) in 0.01s
     Running `target/release/examples/solve`

Board code (blacks/whites):
H8,F6,I7,G6,F8,F9,E7,H4,J6,J4,J5,J8,I4,K6,L6,I2,L5,M7,I8,K8,D5,F4,H3,E2,J11,G11,E10,F13,B11,D12/H7,G7,G9,H6,E8,F7,D6,G5,I5,K5,J3,J7,K4,L7,M6,N7,K7,G4,G8,L8,E4,F3,G2,I10,F11,D9,E12,C10,C12

Board:
---------------
---------------
-----o---------
--xox----------
-o---xo--o-----
--x-o---x------
---x-ox--------
----xoxoooox---
----oxxxoxxxox-
---x-oox-ooox--
---o--x-xoxo---
----xoxooox----
-----x-o-x-----
----o-x-o------
---------------

Black VCF:
	None
	Elapsed: 236.429µs

White VCF:
	8 moves
	G1,G3,E6,H9,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13
	Elapsed: 945.527µs
```

## Release

```
# build
$ wasm-pack build --scope renju-note

# fix manually https://github.com/rustwasm/wasm-pack/issues/837
$ edit pkg/package.json
   "files": [
+    "quintet_bg.js",
     "quintet_bg.wasm",
+    "quintet_bg.wasm.d.ts",
     "quintet.d.ts",
     "quintet.js"
   ],

# publish
$ wasm-pack publish
```

See also: https://developer.mozilla.org/ja/docs/WebAssembly/Rust_to_wasm
