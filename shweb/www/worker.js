import init, { Generator } from "./shweb-wasm/shweb.js";

class SSHKeyWorker {
  /**
   * @type {Generator | null}
   */
  #generator;

  #isRunning = false;
  /**
   * @type {AbortController | null}
   */
  #abortController;

  batchSize;

  constructor(defaultBatchSize = 256) {
    this.batchSize = defaultBatchSize;
  }

  async initialise(config, batchSize = this.batchSize) {
    if (this.#generator) throw new Error("Already initialised");

    await init();

    this.#generator = new Generator(config);
    this.batchSize = batchSize;

    return { success: true, batchSize: this.batchSize };
  }

  async start() {
    if (!this.#generator) throw new Error("Not initialised");
    if (this.#isRunning) throw new Error("Already running");

    this.#isRunning = true;
    this.#abortController = new AbortController();

    try {
      await this.#loop();
    } finally {
      this.#isRunning = false;
    }
  }

  stop() {
    if (!this.#isRunning) return;

    this.#abortController?.abort();
    this.#isRunning = false;
  }

  reset() {
    this.stop();
    this.#generator = null;
  }

  async #loop() {
    const { signal } = this.#abortController;

    try {
      while (this.#isRunning && !signal.aborted) {
        const data = this.#generator.generate_batch(this.batchSize);

        if (data) {
          this.post({
            type: "found",
            data: { publicKey: data[0], privateKey: data[1] },
          });
          this.stop();

          return;
        }

        this.post({ type: "progress", keysGenerated: this.batchSize });
      }
    } catch (err) {
      if (!signal.aborted) this.post({ type: "error", error: err.message });
    }
  }

  post(message) {
    try {
      self.postMessage(message);
    } catch (err) {
      console.error("Failed to post:", err);
    }
  }
}

const worker = new SSHKeyWorker();

const handlers = {
  async init({ config, batchSize }) {
    await worker.initialise(config, batchSize);
    worker.post({ type: "init", batchSize: worker.batchSize });
  },
  start: () => worker.start(),
  stop: () => {
    worker.stop();
    worker.post({ type: "stopped" });
  },
  reset: () => {
    worker.reset();
    worker.post({ type: "reset" });
  },
};

self.addEventListener("message", async ({ data }) => {
  const { type, ...payload } = data;

  try {
    const handler = handlers[type];
    if (!handler) throw new Error(`Unknown message type: ${type}`);

    await handler(payload);
  } catch (error) {
    worker.post({ type: "error", error: error.message });
  }
});

self.addEventListener("error", (event) => {
  worker.post({
    type: "error",
    error: event.message || "Unknown worker error",
  });
});

self.addEventListener("unhandledrejection", (event) => {
  worker.post({
    type: "error",
    error: event.reason?.message || "Unhandled rejection",
  });

  event.preventDefault();
});
