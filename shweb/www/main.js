import init from "./shweb-wasm/shweb.js";
import { setWorkerCountToCpuCores } from "./detect-cpu-cores.js";

const STORAGE_KEY = "shweb-settings";
const UPDATE_INTERVAL = 300;
const DEFAULT_BATCH_SIZE = 256;

const elements = {
  get keywordsInput() {
    return document.getElementById("search-keywords");
  },
  get workersCountInput() {
    return document.getElementById("workers-count");
  },
  get fieldsSelect() {
    return document.getElementById("search-in-fields");
  },
  get anyKeywordRadio() {
    return document.getElementById("any-keyword");
  },
  get allKeywordsRadio() {
    return document.getElementById("all-keywords");
  },
  get anyFieldRadio() {
    return document.getElementById("any-field");
  },
  get allFieldsRadio() {
    return document.getElementById("all-fields");
  },
  get startBtn() {
    return (
      document.querySelector('[data-action="start"]') ||
      document.querySelector('button[type="button"]:nth-of-type(1)')
    );
  },
  get stopBtn() {
    return (
      document.querySelector('[data-action="stop"]') ||
      document.querySelector('button[type="button"]:nth-of-type(2)')
    );
  },
  get resetBtn() {
    return (
      document.querySelector('[data-action="reset"]') ||
      document.querySelector('button[type="button"]:nth-of-type(3)')
    );
  },
  get statusText() {
    return document.querySelector(".status-text");
  },
  get runningTime() {
    return document.querySelector(".stat:nth-child(1) .stat-value");
  },
  get keysGenerated() {
    return document.querySelector(".stat:nth-child(2) .stat-value");
  },
  get keysPerSec() {
    return document.querySelector(".stat:nth-child(3) .stat-value");
  },
  get publicKeyValue() {
    return document.querySelector(".key-info:nth-child(1) .key-value code");
  },
  get privateKeyValue() {
    return document.querySelector(".key-info:nth-child(2) .key-value code");
  },
};

class SSHKeyGeneratorApp {
  #workers = new Set();
  #isGenerating = false;
  #startTime = 0;
  #totalKeysGenerated = 0;
  #statsUpdateInterval = null;
  #abortController = null;

  constructor() {
    this.#bindEventListeners();
    this.#initialise();
  }

  async #initialise() {
    try {
      await init();

      elements.startBtn.disabled = false;

      this.#loadSettings();
      this.#updateStatus("ready");
    } catch (error) {
      console.error("Initialisation failed:", error);
      this.#updateStatus("error");
    }
  }

  #bindEventListeners() {
    elements.startBtn?.addEventListener("click", () => this.start());
    elements.stopBtn?.addEventListener("click", () => this.stop());
    elements.resetBtn?.addEventListener("click", () => this.reset());

    const settingsElements = [
      "keywordsInput",
      "workersCountInput",
      "fieldsSelect",
      "anyKeywordRadio",
      "allKeywordsRadio",
      "anyFieldRadio",
      "allFieldsRadio",
    ];

    settingsElements.forEach((elementKey) => {
      const element = elements[elementKey];
      if (element) {
        const eventType =
          element.type === "select-multiple" ? "change" : "input";
        element.addEventListener(eventType, () => this.#saveSettings());
      }
    });

    window.addEventListener("beforeunload", () => this.#cleanup());
  }

  #updateStatus(status) {
    elements.statusText?.setAttribute("data-status", status);
  }

  #formatTime(seconds) {
    const pad = (num) => Math.floor(num).toString().padStart(2, "0");
    return `${pad(seconds / 3600)}:${pad((seconds % 3600) / 60)}:${pad(
      seconds % 60
    )}`;
  }

  #updateStats() {
    if (!this.#isGenerating) return;

    const elapsedSeconds = (performance.now() - this.#startTime) / 1000;
    const keysPerSec =
      elapsedSeconds > 0
        ? Math.round(this.#totalKeysGenerated / elapsedSeconds)
        : 0;

    elements.runningTime.textContent = this.#formatTime(elapsedSeconds);
    elements.keysGenerated.textContent =
      this.#totalKeysGenerated.toLocaleString();
    elements.keysPerSec.textContent = keysPerSec.toLocaleString();
  }

  #getConfig() {
    const keywords = elements.keywordsInput.value
      .split(",")
      .map((k) => k.trim())
      .filter((k) => k.length > 0);

    const fields = Array.from(elements.fieldsSelect.selectedOptions).map(
      (option) => option.value
    );

    return {
      keywords,
      fields,
      all_keywords: elements.allKeywordsRadio.checked,
      all_fields: elements.allFieldsRadio.checked,
    };
  }

  #validateConfig(config) {
    if (!config.keywords.length) {
      throw new Error("Please enter at least one keyword");
    }
    if (!config.fields.length) {
      throw new Error("Please select at least one field to search");
    }
  }

  #handleWorkerMessage = (event) => {
    const { type, data, keysGenerated, error } = event.data;

    switch (type) {
      case "initialised":
        event.target.postMessage({ type: "start" });
        break;
      case "progress":
        this.#totalKeysGenerated += keysGenerated || 0;
        break;
      case "found":
        this.stop();
        this.#displayResult(data);
        this.#updateStatus("match found");
        break;
      case "error":
        console.error("Worker error:", error);
        this.#updateStatus("error");
        break;
      case "stopped":
        break;
    }
  };

  #displayResult({ publicKey, privateKey }) {
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

      elements.publicKeyValue.textContent = "...";
      elements.privateKeyValue.textContent = "...";

      elements.startBtn.disabled = true;
      elements.stopBtn.disabled = false;
      this.#updateStatus("running");

      await this.#createWorkers(workerCount, config);

      this.#statsUpdateInterval = setInterval(
        () => this.#updateStats(),
        UPDATE_INTERVAL
      );
      this.#updateStats();
    } catch (error) {
      console.error("Start failed:", error);
      this.#updateStatus("error");
      elements.startBtn.disabled = false;
    }
  }

  async #createWorkers(workerCount, config) {
    const workerPromises = Array.from({ length: workerCount }, async () => {
      if (this.#abortController?.signal.aborted) return null;

      try {
        const worker = new Worker("./worker.js", { type: "module" });
        worker.onmessage = this.#handleWorkerMessage;
        worker.onerror = (error) => {
          console.error("Worker error:", error);
          this.#updateStatus("error");
        };

        worker.postMessage({
          type: "init",
          config,
          batchSize: DEFAULT_BATCH_SIZE,
        });

        this.#workers.add(worker);
        return worker;
      } catch (error) {
        console.error("Worker creation failed:", error);
        throw error;
      }
    });

    await Promise.all(workerPromises);
  }

  stop() {
    if (!this.#isGenerating) return;

    this.#isGenerating = false;
    this.#abortController?.abort();

    elements.startBtn.disabled = false;
    elements.stopBtn.disabled = true;

    this.#workers.forEach((worker) => {
      worker.postMessage({ type: "stop" });
      worker.terminate();
    });
    this.#workers.clear();

    if (this.#statsUpdateInterval) {
      clearInterval(this.#statsUpdateInterval);
      this.#statsUpdateInterval = null;
    }

    this.#updateStatus("pending");
  }

  reset() {
    this.stop();

    elements.keywordsInput.value = "";
    setWorkerCountToCpuCores();

    this.#totalKeysGenerated = 0;
    elements.runningTime.textContent = "00:00:00";
    elements.keysGenerated.textContent = "0";
    elements.keysPerSec.textContent = "0";

    elements.publicKeyValue.textContent = "...";
    elements.privateKeyValue.textContent = "...";

    this.#updateStatus("pending");
    this.#clearSettings();
  }

  #saveSettings() {
    try {
      const settings = {
        keywords: elements.keywordsInput.value,
        workersCount: elements.workersCountInput.value,
        fields: Array.from(elements.fieldsSelect.selectedOptions).map(
          (o) => o.value
        ),
        keywordMatching: elements.allKeywordsRadio.checked ? "all" : "any",
        fieldMatching: elements.allFieldsRadio.checked ? "all" : "any",
      };

      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch (error) {
      console.warn("Failed to save settings:", error);
    }
  }

  #loadSettings() {
    try {
      const settings = JSON.parse(
        sessionStorage.getItem(STORAGE_KEY) || "null"
      );
      if (!settings) return;

      elements.keywordsInput.value = settings.keywords || "";
      elements.workersCountInput.value =
        settings.workersCount || elements.workersCountInput.value;

      Array.from(elements.fieldsSelect.options).forEach((option) => {
        option.selected = settings.fields?.includes(option.value) || false;
      });

      const keywordRadios =
        settings.keywordMatching === "all"
          ? [elements.allKeywordsRadio, elements.anyKeywordRadio]
          : [elements.anyKeywordRadio, elements.allKeywordsRadio];

      keywordRadios[0].checked = true;
      keywordRadios[1].checked = false;

      const fieldRadios =
        settings.fieldMatching === "all"
          ? [elements.allFieldsRadio, elements.anyFieldRadio]
          : [elements.anyFieldRadio, elements.allFieldsRadio];

      fieldRadios[0].checked = true;
      fieldRadios[1].checked = false;
    } catch (error) {
      console.warn("Failed to load settings:", error);
    }
  }

  #clearSettings() {
    try {
      sessionStorage.removeItem(STORAGE_KEY);
    } catch (error) {
      console.warn("Failed to clear settings:", error);
    }
  }

  #cleanup() {
    this.stop();
  }
}

new SSHKeyGeneratorApp();
