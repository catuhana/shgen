import init, { Generator } from "../pkg/shweb.js";

class SSHKeyWorker {
  constructor() {
    this.generator = null;

    this.isRunning = false;

    this.batchSize = 256;

    this.abortController = null;
  }

  async initialize(config, batchSize = 256) {
    try {
      await init();

      this.generator = new Generator(config);
      this.batchSize = batchSize;

      return { success: true };
    } catch (error) {
      throw new Error(`Initialization failed: ${error.message}`);
    }
  }

  start() {
    if (this.isRunning) {
      throw new Error("Generation already running");
    }

    this.isRunning = true;
    this.abortController = new AbortController();

    return this.generateLoop();
  }

  stop() {
    this.isRunning = false;
    this.abortController?.abort();
  }

  async generateLoop() {
    const { signal } = this.abortController;

    try {
      while (this.isRunning && !signal.aborted) {
        const result = await this.generateBatch();

        if (result.found) {
          this.postMessage({
            type: "found",
            data: result.data,
          });

          break;
        }

        this.postMessage({
          type: "progress",
          keysGenerated: this.batchSize,
        });

        await this.yieldControl();
      }
    } catch (error) {
      if (!signal.aborted) {
        this.postMessage({
          type: "error",
          error: error.message,
          stack: error.stack,
        });
      }
    } finally {
      this.isRunning = false;
    }
  }

  async generateBatch() {
    if (!this.generator) {
      throw new Error("Generator not initialized");
    }

    return new Promise((resolve, reject) => {
      try {
        const result = this.generator.generate_batch(this.batchSize);

        resolve({
          found: result !== null && result !== undefined,
          data: result,
        });
      } catch (error) {
        reject(new Error(`Batch generation failed: ${error.message}`));
      }
    });
  }

  yieldControl() {
    return new Promise((resolve) => {
      queueMicrotask(resolve);
    });
  }

  postMessage(message) {
    self.postMessage(message);
  }
}

const worker = new SSHKeyWorker();

self.addEventListener("message", async (event) => {
  const { type, config, batchSize } = event.data;

  try {
    switch (type) {
      case "init":
        await worker.initialize(config, batchSize);

        worker.postMessage({
          type: "initialized",
          batchSize: worker.batchSize,
        });

        break;
      case "start":
        if (!worker.generator) {
          throw new Error("Worker not initialized. Call init first.");
        }

        await worker.start();

        break;
      case "stop":
        worker.stop();
        worker.postMessage({ type: "stopped" });

        break;
      default:
        throw new Error(`Unknown message type: ${type}`);
    }
  } catch (error) {
    worker.postMessage({
      type: "error",
      error: error.message,
      stack: error.stack,
    });
  }
});

self.addEventListener("error", (error) => {
  worker.postMessage({
    type: "error",
    error: error.message,
    filename: error.filename,
    lineno: error.lineno,
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
