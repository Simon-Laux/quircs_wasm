const path = require("path");
const { readFile } = require("fs/promises");

let wasmPlugin = {
  name: "wasm",
  setup(build) {
    // Resolve ".wasm" files to a path with a namespace
    build.onResolve({ filter: /\.wasm$/ }, (args) => {
      if (args.resolveDir === "") {
        return; // Ignore unresolvable paths
      }
      return {
        path: path.isAbsolute(args.path)
          ? args.path
          : path.join(args.resolveDir, args.path),
        namespace: "wasm-binary",
      };
    });

    // Virtual modules in the "wasm-binary" namespace contain the
    // actual bytes of the WebAssembly file. This uses esbuild's
    // built-in "binary" loader instead of manually embedding the
    // binary data inside JavaScript code ourselves.
    build.onLoad({ filter: /.*/, namespace: "wasm-binary" }, async (args) => ({
      contents: await readFile(args.path),
      loader: "binary",
    }));
  },
};

require("esbuild")
  .build({
    entryPoints: ["index.tsx"],
    bundle: true,
    outfile: "bundle.js",
    plugins: [wasmPlugin],
  })
  .then((value) => {
    console.log(value);
  })
  .catch((err) => {
    console.log(err);
    process.exit(1);
  });
