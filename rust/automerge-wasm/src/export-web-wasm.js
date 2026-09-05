// This file is inserted into ./web by the build script

import init from "./automerge_wasm.js";
await init(new URL("./automerge_wasm_bg.wasm", import.meta.url));
export * from "./automerge_wasm.js";
