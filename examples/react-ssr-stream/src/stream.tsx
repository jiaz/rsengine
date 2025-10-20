import { Writable } from "node:stream";
import { renderToPipeableStream } from "react-dom/server";
import App from "./App";

export type StreamContext = {
  write(chunk: string): void;
  close?: () => void;
  flush?: () => void;
  onError?: (error: unknown) => void;
  registerAbort?: (abort: () => void) => void;
};

/**
 * Entry point consumed by the rsengine runtime.
 * It wires React's `renderToPipeableStream` to the host's streaming bridge.
 */
export async function stream(context: StreamContext): Promise<void> {
  return new Promise((resolve, reject) => {
    const { pipe, abort } = renderToPipeableStream(<App />, {
      onShellReady() {
        context.flush?.();

        const writable = new Writable({
          write(chunk, _encoding, callback) {
            try {
              const payload =
                typeof chunk === "string" ? chunk : chunk.toString("utf8");
              context.write(payload);
              callback();
            } catch (error) {
              callback(error as Error);
            }
          },
          final(callback) {
            try {
              context.close?.();
              resolve();
              callback();
            } catch (error) {
              callback(error as Error);
            }
          },
        });

        writable.on("error", (error) => {
          context.onError?.(error);
          reject(error);
        });

        pipe(writable);
      },
      onShellError(error) {
        context.onError?.(error);
        reject(error);
      },
      onError(error) {
        context.onError?.(error);
      },
    });

    context.registerAbort?.(abort);
  });
}
