/**
 * Type declaration for the #automerge subpath import.
 *
 * The default "node" entrypoint loads WASM via fs.readFileSync(__dirname + "..."),
 * which breaks in `bun build --compile` standalone binaries because __dirname
 * gets hardcoded to the build machine's absolute path.
 *
 * The #automerge import (defined in package.json "imports") resolves to the
 * base64 entrypoint which embeds WASM inline and works everywhere.
 */
declare module "#automerge" {
  export * from "@automerge/automerge";
}
