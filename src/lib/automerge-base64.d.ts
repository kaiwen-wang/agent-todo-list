/**
 * Type declaration for the #automerge subpath import.
 *
 * We use the "fullfat_node" entrypoint which uses Node-compatible WASM loading
 * (CJS require) and avoids the deprecated `initSync()` parameter warning that
 * the "fullfat_base64" (web) entrypoint triggers.
 *
 * The #automerge path alias is defined in tsconfig.json "paths" and resolved
 * by Bun at runtime.
 */
declare module "#automerge" {
  export * from "@automerge/automerge";
}
