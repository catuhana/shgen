export function setWorkerCountToCpuCores() {
  const amountOfThreads = navigator.hardwareConcurrency;
  if (amountOfThreads) {
    /** @type {HTMLSpanElement} */ (
      document.querySelector("span.cpu-cores-count")
    ).textContent = String(amountOfThreads);
    /** @type {HTMLInputElement} */ (
      document.querySelector("input[name='workers-count']")
    ).value = String(amountOfThreads);
  }
}

setWorkerCountToCpuCores();
