# quintet

Renju library with wasm-pack

# Note

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
