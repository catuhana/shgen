import init, { Generator, SearchFields } from "../shweb-wasm/shweb.js";

// Keep it synced with shweb/src/lib.rs
const BATCH_SIZE = (8 * 1024) / 32;

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

  async initialise(config) {
    if (this.#generator) throw new Error("Already initialised");

    await init();

    const fieldMap = {
      "private-key": SearchFields.PrivateKey,
      "public-key": SearchFields.PublicKey,
      "sha1-fingerprint": SearchFields.Sha1Fingerprint,
      "sha256-fingerprint": SearchFields.Sha256Fingerprint,
      "sha384-fingerprint": SearchFields.Sha384Fingerprint,
      "sha512-fingerprint": SearchFields.Sha512Fingerprint,
    };
    const fields = config.search.fields.map((field) => fieldMap[field]);

    this.#generator = new Generator(
      config.keywords,
      fields,
      config.search.matching["all-keywords"],
      config.search.matching["all-fields"]
    );

    return { success: true };
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
        const data = this.#generator.generateBatch();

        if (data) {
          this.post({
            type: "found",
            data: { publicKey: data[0], privateKey: data[1] },
          });
          this.stop();

          return;
        }

        this.post({ type: "progress", keysGenerated: BATCH_SIZE });
      }
    } catch (error) {
      if (!signal.aborted) this.post({ type: "error", error: error.message });
    }
  }

  post(message) {
    try {
      self.postMessage(message);
    } catch (error) {
      console.error("Failed to post:", error);
    }
  }
}

const worker = new SSHKeyWorker();

const handlers = {
  async init({ config }) {
    await worker.initialise(config);
    worker.post({ type: "init", batchSize: BATCH_SIZE });
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
