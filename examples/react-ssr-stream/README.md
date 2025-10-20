# React SSR Streaming Bundle

This example project produces a JavaScript bundle that exports a `stream` handler compatible with the `rsengine` runtime. The handler uses React 18's `renderToPipeableStream` API to stream HTML chunks back to the server as they become ready.

## Prerequisites

- Node.js 18+
- npm (or another Node package manager)

## Install dependencies

```bash
cd examples/react-ssr-stream
npm install
```

## Build the bundle

```bash
npm run build
```

The compiled bundle is written to `dist/app.bundle.js`. Point the rsengine server at this path:

```bash
cargo run -p server -- --bundle ./examples/react-ssr-stream/dist/app.bundle.js
```

## Handler contract

The generated module exports a single async function:

```ts
export async function stream(context: StreamContext): Promise<void>;
```

The `context` object is provided by rsengine and exposes a minimal streaming bridge:

- `write(chunk: string)` – send an HTML fragment to the client.
- `close()` – complete the stream (optional).
- `flush()` – hint that buffered data should be flushed (optional).
- `onError(error)` – surface rendering errors back to the host (optional callback).
- `registerAbort(abort)` – allow the host to cancel the render (optional).

The included React example renders a suspenseful component so you can observe multiple chunks arriving in the response stream.

## Modifying the app

- `src/App.tsx` contains the React component tree.
- `src/stream.tsx` is the bundle entry point that wires React's streaming renderer to the `context` bridge.
- `build.mjs` drives esbuild to produce an ESM bundle targeted at Node runtimes.

Adjust or extend these files to fit your application. When ready, rebuild the bundle and restart `rsengine` with the new path.
