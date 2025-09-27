import init from "../pkg/shweb.js";

import { setWorkerCountToCpuCores } from "./detect-cpu-cores.js";

const state = {
  workers: [],
  isGenerating: false,
  startTime: 0,
  totalKeysGenerated: 0,
  statusInterval: null,
};

const elements = {
  keywordsInput: document.getElementById("search-keywords"),
  workersCountInput: document.getElementById("workers-count"),
  fieldsSelect: document.getElementById("search-in-fields"),
  anyKeywordRadio: document.getElementById("any-keyword"),
  allKeywordsRadio: document.getElementById("all-keywords"),
  anyFieldRadio: document.getElementById("any-field"),
  allFieldsRadio: document.getElementById("all-fields"),
  startBtn: document.querySelector('button[type="button"]:nth-of-type(1)'),
  stopBtn: document.querySelector('button[type="button"]:nth-of-type(2)'),
  resetBtn: document.querySelector('button[type="button"]:nth-of-type(3)'),
  statusText: document.querySelector(".status-text"),
  runningTime: document.querySelector(".stat:nth-child(1) .stat-value"),
  keysGenerated: document.querySelector(".stat:nth-child(2) .stat-value"),
  keysPerSec: document.querySelector(".stat:nth-child(3) .stat-value"),
  publicKeyValue: document.querySelector(
    ".key-info:nth-child(1) .key-value code"
  ),
  privateKeyValue: document.querySelector(
    ".key-info:nth-child(2) .key-value code"
  ),
};

async function initialiseShweb() {
  try {
    await init();

    elements.startBtn.disabled = false;
    loadSettingsFromLocalStorage();
  } catch (error) {
    updateStatus("error");
  }
}

function updateStatus(status = "pending") {
  elements.statusText.setAttribute("data-status", status);
}

function formatTime(seconds) {
  const h = Math.floor(seconds / 3600)
    .toString()
    .padStart(2, "0");
  const m = Math.floor((seconds % 3600) / 60)
    .toString()
    .padStart(2, "0");
  const s = Math.floor(seconds % 60)
    .toString()
    .padStart(2, "0");

  return `${h}:${m}:${s}`;
}

function updateStats() {
  if (!state.isGenerating) return;

  const elapsedSeconds = (performance.now() - state.startTime) / 1000;
  const keysPerSec = Math.round(state.totalKeysGenerated / elapsedSeconds);

  elements.runningTime.textContent = formatTime(elapsedSeconds);
  elements.keysGenerated.textContent =
    state.totalKeysGenerated.toLocaleString();
  elements.keysPerSec.textContent = keysPerSec.toLocaleString();
}

function getConfig() {
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

function validateConfig(config) {
  if (config.keywords.length === 0) {
    throw new Error("Please enter at least one keyword");
  }

  if (config.fields.length === 0) {
    throw new Error("Please select at least one field to search");
  }
}

function handleWorkerMessage(event) {
  const { type, data, keysGenerated, error } = event.data;

  switch (type) {
    case "initialized":
      event.target.postMessage({ type: "start" });
      break;
    case "progress":
      state.totalKeysGenerated += keysGenerated || 0;
      break;
    case "found":
      stopGeneration();

      displayResult(data);
      updateStatus("match found");

      break;
    case "error":
      updateStatus(`Error: ${error}`, "error");
      break;
  }
}

function displayResult(result) {
  elements.publicKeyValue.textContent = result[0];
  elements.privateKeyValue.textContent = result[1];
}

async function startGeneration() {
  try {
    const config = getConfig();
    validateConfig(config);

    const workerCount = Math.max(1, parseInt(elements.workersCountInput.value));

    state.isGenerating = true;
    state.startTime = performance.now();
    state.totalKeysGenerated = 0;

    elements.publicKeyValue.textContent = "...";
    elements.privateKeyValue.textContent = "...";

    elements.startBtn.disabled = true;
    elements.stopBtn.disabled = false;
    updateStatus("running");

    for (let i = 0; i < workerCount; i++) {
      const worker = new Worker("./worker.js", { type: "module" });

      worker.onmessage = handleWorkerMessage;
      worker.postMessage({
        type: "init",
        config,
        batchSize: 256,
      });

      state.workers.push(worker);
    }

    state.statusInterval = setInterval(updateStats, 1000);
    updateStats();
  } catch (error) {
    updateStatus("error");
    elements.startBtn.disabled = false;
  }
}

function stopGeneration() {
  if (!state.isGenerating) return;

  state.isGenerating = false;

  elements.startBtn.disabled = false;
  elements.stopBtn.disabled = true;

  state.workers.forEach((worker) => {
    worker.postMessage({ type: "stop" });
    worker.terminate();
  });
  state.workers = [];

  if (state.statusInterval) {
    clearInterval(state.statusInterval);
    state.statusInterval = null;
  }

  updateStatus("pending");
}

function resetApplication() {
  stopGeneration();

  elements.keywordsInput.value = "";
  setWorkerCountToCpuCores();

  state.totalKeysGenerated = 0;
  elements.runningTime.textContent = "00:00:00";
  elements.keysGenerated.textContent = "0";
  elements.keysPerSec.textContent = "0";

  elements.publicKeyValue.textContent = "...";
  elements.privateKeyValue.textContent = "...";

  updateStatus("pending");
  clearSettingsFromLocalStorage();
}

elements.startBtn?.addEventListener("click", startGeneration);
elements.stopBtn?.addEventListener("click", stopGeneration);
elements.resetBtn?.addEventListener("click", resetApplication);

elements.keywordsInput.addEventListener("input", saveSettingsToLocalStorage);
elements.workersCountInput.addEventListener(
  "input",
  saveSettingsToLocalStorage
);
elements.fieldsSelect.addEventListener("change", saveSettingsToLocalStorage);
elements.anyKeywordRadio.addEventListener("change", saveSettingsToLocalStorage);
elements.allKeywordsRadio.addEventListener(
  "change",
  saveSettingsToLocalStorage
);
elements.anyFieldRadio.addEventListener("change", saveSettingsToLocalStorage);
elements.allFieldsRadio.addEventListener("change", saveSettingsToLocalStorage);

function saveSettingsToLocalStorage() {
  const settings = {
    keywords: elements.keywordsInput.value,
    workersCount: elements.workersCountInput.value,
    fields: Array.from(elements.fieldsSelect.selectedOptions).map(
      (o) => o.value
    ),
    keywordMatching: elements.allKeywordsRadio.checked ? "all" : "any",
    fieldMatching: elements.allFieldsRadio.checked ? "all" : "any",
  };

  localStorage.setItem("settings", JSON.stringify(settings));
}

function loadSettingsFromLocalStorage() {
  const settings = JSON.parse(localStorage.getItem("settings") || "null");
  if (!settings) return;

  elements.keywordsInput.value = settings.keywords || "";
  elements.workersCountInput.value =
    settings.workersCount || elements.workersCountInput.value;

  Array.from(elements.fieldsSelect.options).forEach((option) => {
    option.selected = settings.fields?.includes(option.value) || false;
  });

  if (settings.keywordMatching === "all") {
    elements.allKeywordsRadio.checked = true;
    elements.anyKeywordRadio.checked = false;
  } else {
    elements.anyKeywordRadio.checked = true;
    elements.allKeywordsRadio.checked = false;
  }
  if (settings.fieldMatching === "all") {
    elements.allFieldsRadio.checked = true;
    elements.anyFieldRadio.checked = false;
  } else {
    elements.anyFieldRadio.checked = true;
    elements.allFieldsRadio.checked = false;
  }
}

function clearSettingsFromLocalStorage() {
  localStorage.removeItem("settings");
}

window.addEventListener("beforeunload", () => {
  state.workers.forEach((worker) => worker.terminate());
});

initialiseShweb();
