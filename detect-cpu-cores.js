export function setWorkerCountToCpuCores() {
  const amountOfThreads = navigator.hardwareConcurrency;
  if (amountOfThreads) {
    document.querySelector("span.cpu-cores-count").textContent =
      amountOfThreads;
    document.querySelector("input[name='workers-count'").value =
      amountOfThreads;
  }
}

setWorkerCountToCpuCores();
