## farm-plugin-wasm

Add WebAssembly integration to farm

### Installation

```bash
pnpm add -D farm-plugin-wasm
```

### Usage

```ts
import { defineConfig } from "@farmfe/core";
import wasm from 'farm-plugin-wasm';

interface Options {
  /* Your options here */
  /**
   * Whether to split the WASM file
   */
  isolate?: boolean // default: false
}

export default defineConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    output: {
      filename: 'assets/[ext]/[name].[hash].[ext]',
      assetsFilename: 'static/[resourceName].[ext]'
    },
    persistentCache: false,
    progress: false,
  },
  plugins: [
    wasm({
      isolate: false
    })
  ]
});
```

### Notes
If you set the ```isolate```option to ```true```, the plugin will use ```top level await``` to get the ```wasm``` file.