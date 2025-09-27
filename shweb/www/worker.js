import init, { Generator } from "../pkg/shweb.js";

class SSHKeyWorker {
  #generator = null;
  #isRunning = false;
  batchSize = 256;
  #abortController = null;

  async initialise(config, batchSize = 256) {
    if (this.#generator) {
      throw new Error("Worker already initialised");
    }

    try {
      await init();

      this.#generator = new Generator(config);
      this.batchSize = batchSize;

      return { success: true, batchSize: this.batchSize };
    } catch (error) {
      throw new Error(`Initialisation failed: ${error.message}`);
    }
  }

  async start() {
    if (!this.#generator) {
      throw new Error("Worker not initialised. Call initialise first.");
    }

    if (this.#isRunning) {
      throw new Error("Generation already running");
    }

    this.#isRunning = true;
    this.#abortController = new AbortController();

    try {
      await this.#generateLoop();
    } finally {
      this.#isRunning = false;
    }
  }

  stop() {
    if (!this.#isRunning) return;

    this.#isRunning = false;
    this.#abortController?.abort();
  }

  reset() {
    this.stop();

    this.#generator = null;
  }

  async #generateLoop() {
    const { signal } = this.#abortController;

    try {
      while (this.#isRunning && !signal.aborted) {
        const result = this.#generateBatch(); // Synchronous, no await

        if (result.found) {
          this.postMessage({
            type: "found",
            data: {
              publicKey: result.data[0],
              privateKey: result.data[1],
            },
          });
          break;
        }

        this.postMessage({
          type: "progress",
          keysGenerated: this.batchSize,
        });
      }
    } catch (error) {
      if (!signal.aborted) {
        this.postMessage({
          type: "error",
          error: error.message,
        });
      }
    }
  }

  #generateBatch() {
    if (!this.#generator) {
      throw new Error("Generator not initialised");
    }

    try {
      const result = this.#generator.generate_batch(this.batchSize);

      return {
        found: result !== null && result !== undefined,
        data: result,
      };
    } catch (error) {
      throw new Error(`Batch generation failed: ${error.message}`);
    }
  }

  postMessage(message) {
    try {
      self.postMessage(message);
    } catch (error) {
      console.error("Failed to post message:", error);
    }
  }
}

const worker = new SSHKeyWorker();

const messageHandlers = {
  async init({ config, batchSize }) {
    await worker.initialise(config, batchSize);

    worker.postMessage({
      type: "initialised",
      batchSize: worker.batchSize,
    });
  },

  async start() {
    await worker.start();
  },

  stop() {
    worker.stop();
    worker.postMessage({ type: "stopped" });
  },

  reset() {
    worker.reset();
    worker.postMessage({ type: "reset" });
  },
};

self.addEventListener("message", async (event) => {
  const { type, ...data } = event.data;

  try {
    const handler = messageHandlers[type];
    if (!handler) {
      throw new Error(`Unknown message type: ${type}`);
    }

    await handler(data);
  } catch (error) {
    worker.postMessage({
      type: "error",
      error: error.message,
      stack: error.stack,
    });
  }
});

self.addEventListener("error", (event) => {
  worker.postMessage({
    type: "error",
    error: event.message || "Unknown worker error",
    filename: event.filename,
    lineno: event.lineno,
    colno: event.colno,
  });
});

self.addEventListener("unhandledrejection", (event) => {
  worker.postMessage({
    type: "error",
    error: event.reason?.message || "Unhandled promise rejection",
    stack: event.reason?.stack,
  });

  event.preventDefault();
});
