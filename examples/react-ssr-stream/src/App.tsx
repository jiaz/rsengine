import { Suspense } from "react";

function createDeferredResource<T>(value: T, delayMs = 200): { read(): T } {
  let status: "pending" | "resolved" = "pending";
  let stored: T;
  const promise = new Promise<void>((resolve) => {
    setTimeout(() => {
      stored = value;
      status = "resolved";
      resolve();
    }, delayMs);
  });

  return {
    read(): T {
      if (status === "pending") {
        throw promise;
      }
      return stored!;
    },
  };
}

const quoteResource = createDeferredResource(
  "Streaming is working! This line was resolved on the server.",
  300,
);

function Quote() {
  const text = quoteResource.read();
  return <p className="quote">{text}</p>;
}

const App = () => (
  <div className="app">
    <header>
      <h1>React 18 Streaming Demo</h1>
      <p>
        The first bytes reach the client immediately, while the quote waits on a
        deferred promise to illustrate streaming behaviour.
      </p>
    </header>
    <Suspense fallback={<p className="loading">Fetching inspirational quote…</p>}>
      <Quote />
    </Suspense>
  </div>
);

export default App;
