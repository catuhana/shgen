import { setWorkerCountToCpuCores } from "./detect-cpu-cores.js";

const STORAGE_KEY = "shweb-settings";
const UPDATE_INTERVAL = 300;
const DEFAULT_BATCH_SIZE = 256;

const $ = (selector) => document.querySelector(selector);

const elements = {
  keywordsInput: $("#search-keywords"),
  workersCountInput: $("#workers-count"),
  fieldsSelect: $("#search-in-fields"),
  anyKeywordRadio: $("#any-keyword"),
  allKeywordsRadio: $("#all-keywords"),
  anyFieldRadio: $("#any-field"),
  allFieldsRadio: $("#all-fields"),
  startButton: $('[data-action="start"]'),
  stopButton: $('[data-action="stop"]'),
  resetButton: $('[data-action="reset"]'),
  statusText: $(".status-text"),
  runningTime: $(".stat:nth-child(1) .stat-value"),
  keysGenerated: $(".stat:nth-child(2) .stat-value"),
  keysPerSec: $(".stat:nth-child(3) .stat-value"),
  publicKeyValue: $(".key-info:nth-child(1) .key-value code"),
  privateKeyValue: $(".key-info:nth-child(2) .key-value code"),
};

class Shweb {
  #workers = new Set();

  #isGenerating = false;
  #startTime = 0;
  #totalKeysGenerated = 0;

  /**
   * @type {number | null}
   */
  #statsUpdateInterval;
  /**
   * @type {AbortController | null}
   */
  #abortController;

  constructor() {
    this.#bindEvents();
    this.#init();
  }

  async #init() {
    try {
      elements.startButton.disabled = false;

      this.#loadSettings();
      this.#setStatus("ready");
    } catch (error) {
      console.error("Init failed:", error);
      this.#setStatus("error");
    }
  }

  #bindEvents() {
    elements.startButton?.addEventListener("click", () => this.start());
    elements.stopButton?.addEventListener("click", () => this.stop());
    elements.resetButton?.addEventListener("click", () => this.reset());

    const settingsInputs = [
      elements.keywordsInput,
      elements.workersCountInput,
      elements.fieldsSelect,
      elements.anyKeywordRadio,
      elements.allKeywordsRadio,
      elements.anyFieldRadio,
      elements.allFieldsRadio,
    ];

    for (const element of settingsInputs) {
      if (!element) continue;

      const event = element.type === "select-multiple" ? "change" : "input";
      element.addEventListener(event, () => this.#saveSettings());
    }

    window.addEventListener("beforeunload", () => this.stop());
  }

  #setStatus(status) {
    elements.statusText?.setAttribute("data-status", status);
  }

  #formatTime(seconds) {
    const pad = (n) => String(Math.floor(n)).padStart(2, "0");

    return `${pad(seconds / 3600)}:${pad((seconds % 3600) / 60)}:${pad(
      seconds % 60
    )}`;
  }

  #updateStats() {
    if (!this.#isGenerating) return;

    const elapsed = (performance.now() - this.#startTime) / 1000;
    const rate = elapsed ? Math.round(this.#totalKeysGenerated / elapsed) : 0;

    elements.runningTime.textContent = this.#formatTime(elapsed);
    elements.keysGenerated.textContent =
      this.#totalKeysGenerated.toLocaleString();
    elements.keysPerSec.textContent = rate.toLocaleString();
  }

  #getConfig() {
    const keywords = elements.keywordsInput.value
      .split(",")
      .map((keyword) => keyword.trim())
      .filter(Boolean);

    const fields = [...elements.fieldsSelect.selectedOptions].map(
      (option) => option.value
    );

    return {
      keywords,
      search: {
        fields,
        matching: {
          "all-keywords": elements.allKeywordsRadio.checked,
          "all-fields": elements.allFieldsRadio.checked,
        },
      },
    };
  }

  #validateConfig({ keywords, search: { fields } }) {
    if (!keywords.length) throw new Error("Enter at least one keyword");
    if (!fields.length) throw new Error("Select at least one field");
  }

  #handleWorkerMessage = ({ data, target }) => {
    const { type, data: payload, keysGenerated, error } = data;

    switch (type) {
      case "init":
        target.postMessage({ type: "start" });
        break;
      case "progress":
        this.#totalKeysGenerated += keysGenerated ?? 0;
        break;
      case "found":
        this.stop();

        this.#showResult(payload);
        this.#setStatus("match found");

        break;
      case "error":
        console.error("Worker error:", error);
        this.#setStatus("error");

        break;
      case "stopped":
        if (this.#isGenerating) this.stop();
        break;
      case "reset":
        if (this.#isGenerating || this.#totalKeysGenerated > 0) this.reset();
        break;
    }
  };

  #showResult({ publicKey, privateKey }) {
    elements.publicKeyValue.textContent = publicKey || "Error loading key";
    elements.privateKeyValue.textContent = privateKey || "Error loading key";
  }

  async start() {
    try {
      const config = this.#getConfig();
      this.#validateConfig(config);

      const workerCount = Math.max(
        1,
        parseInt(elements.workersCountInput.value) || 1
      );

      this.#isGenerating = true;
      this.#startTime = performance.now();
      this.#totalKeysGenerated = 0;
      this.#abortController = new AbortController();

      elements.publicKeyValue.textContent =
        elements.privateKeyValue.textContent = "...";
      elements.startButton.disabled = true;
      elements.stopButton.disabled = false;

      this.#setStatus("running");

      await this.#spawnWorkers(workerCount, config);

      this.#statsUpdateInterval = setInterval(
        () => this.#updateStats(),
        UPDATE_INTERVAL
      );
      this.#updateStats();
    } catch (err) {
      console.error("Start failed:", err);
      this.#setStatus("error");

      elements.startButton.disabled = false;
    }
  }

  async #spawnWorkers(count, config) {
    const tasks = Array.from({ length: count }, async () => {
      if (this.#abortController?.signal.aborted) return;

      const worker = new Worker(new URL("./worker.js", import.meta.url), {
        type: "module",
      });
      worker.onmessage = this.#handleWorkerMessage;
      worker.onerror = (error) => {
        console.error("Worker error:", error);

        this.stop();
        this.#setStatus("error");
      };

      worker.postMessage({
        type: "init",
        config,
        batchSize: DEFAULT_BATCH_SIZE,
      });
      this.#workers.add(worker);
    });

    await Promise.all(tasks);
  }

  stop() {
    if (!this.#isGenerating) return;

    this.#isGenerating = false;
    this.#abortController?.abort();

    elements.startButton.disabled = false;
    elements.stopButton.disabled = true;

    for (const worker of this.#workers) {
      worker.postMessage({ type: "stop" });
      worker.terminate();
    }
    this.#workers.clear();

    clearInterval(this.#statsUpdateInterval);
    this.#statsUpdateInterval = null;

    this.#setStatus("ready");
  }

  reset() {
    this.stop();

    elements.keywordsInput.value = "";
    setWorkerCountToCpuCores();

    this.#totalKeysGenerated = 0;

    elements.runningTime.textContent = "00:00:00";
    elements.keysGenerated.textContent = "0";
    elements.keysPerSec.textContent = "0";
    elements.publicKeyValue.textContent = elements.privateKeyValue.textContent =
      "...";

    elements.anyKeywordRadio.checked = elements.allFieldsRadio.checked = true;
    elements.allKeywordsRadio.checked = elements.anyFieldRadio.checked = false;

    this.#setStatus("ready");

    localStorage.removeItem(STORAGE_KEY);
  }

  #saveSettings() {
    const settings = {
      keywords: elements.keywordsInput.value,
      workersCount: elements.workersCountInput.value,
      fields: [...elements.fieldsSelect.selectedOptions].map((o) => o.value),
      keywordMatching: elements.allKeywordsRadio.checked ? "all" : "any",
      fieldMatching: elements.allFieldsRadio.checked ? "all" : "any",
    };

    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }

  #loadSettings() {
    const settings = JSON.parse(localStorage.getItem(STORAGE_KEY) || "null");
    if (!settings) return;

    elements.keywordsInput.value = settings.keywords ?? "";
    elements.workersCountInput.value =
      settings.workersCount ?? elements.workersCountInput.value;

    for (const opt of elements.fieldsSelect.options) {
      opt.selected = settings.fields?.includes(opt.value) ?? false;
    }

    (settings.keywordMatching === "all"
      ? elements.allKeywordsRadio
      : elements.anyKeywordRadio
    ).checked = true;

    (settings.fieldMatching === "all"
      ? elements.allFieldsRadio
      : elements.anyFieldRadio
    ).checked = true;
  }
}

new Shweb();
