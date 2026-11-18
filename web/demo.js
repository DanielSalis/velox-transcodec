"use strict";

const examples = [
  { name: "entrevista_01.mov", codec: "ProRes", resolution: "3840×2160", duration: "02:14", fps: "29,97", bitrate: "428 Mb/s", size: "7,2 GB" },
  { name: "campo_aberto.mp4", codec: "HEVC", resolution: "3840×2160", duration: "01:38", fps: "59,94", bitrate: "53 Mb/s", size: "652 MB" },
  { name: "captura_tela.webm", codec: "VP9", resolution: "2560×1440", duration: "00:52", fps: "60", bitrate: "14 Mb/s", size: "91 MB" },
  { name: "cidade_noite.mkv", codec: "AV1", resolution: "1920×1080", duration: "00:46", fps: "24", bitrate: "8 Mb/s", size: "46 MB" },
  { name: "bastidores.mp4", codec: "H.264", resolution: "1920×1080", duration: "01:20", fps: "30", bitrate: "19 Mb/s", size: "190 MB" },
];

const encoders = {
  vaapi: { quality: "Qualidade constante", speeds: ["Automático"], defaultSpeed: "Automático", fps: 112, duration: 5500 },
  nvenc: { quality: "Qualidade constante", speeds: ["P1", "P2", "P3", "P4", "P5", "P6", "P7"], defaultSpeed: "P4", fps: 144, duration: 4800 },
  x264: {
    quality: "CRF",
    speeds: ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"],
    defaultSpeed: "medium",
    fps: 62,
    duration: 7000,
  },
};

const fileList = document.querySelector("#file-list");
const fileCount = document.querySelector("#file-count");
const addFile = document.querySelector("#add-file");
const encoder = document.querySelector("#encoder");
const preset = document.querySelector("#preset");
const quality = document.querySelector("#quality");
const qualityValue = document.querySelector("#quality-value");
const progress = document.querySelector("#queue-progress");
const percent = document.querySelector("#queue-percent");
const status = document.querySelector("#queue-status");
const startButton = document.querySelector("#start-queue");
const outputControls = document.querySelector("#output-controls");
const dialog = document.querySelector("#instalacao");
const menu = document.querySelector(".menu-toggle");
const siteNav = document.querySelector("#site-nav");

let fileTotal = 3;
let selectedIndex = 0;
let run = null;
const rows = [];

function fileIcon() {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
  svg.classList.add("icon");
  svg.setAttribute("aria-hidden", "true");
  use.setAttribute("href", "#i-video");
  svg.append(use);
  return svg;
}

function addRow(file, index) {
  const row = fileList.insertRow();
  const button = document.createElement("button");
  const name = document.createElement("span");
  const jobProgress = document.createElement("span");
  button.className = "file-button";
  button.type = "button";
  button.title = file.name;
  button.setAttribute("aria-pressed", "false");
  button.addEventListener("click", () => selectFile(index));
  name.textContent = file.name;
  jobProgress.className = "file-progress";
  jobProgress.hidden = true;
  button.append(fileIcon(), name);
  row.insertCell().append(button, jobProgress);
  [file.codec, file.resolution, file.duration].forEach(value => {
    row.insertCell().textContent = value;
  });
  rows.push({ button, jobProgress });
}

function selectFile(index) {
  rows[selectedIndex].button.setAttribute("aria-pressed", "false");
  selectedIndex = index;
  rows[index].button.setAttribute("aria-pressed", "true");
  const { fps, bitrate, size } = examples[index];
  document.querySelector("#file-fps").textContent = fps;
  document.querySelector("#file-bitrate").textContent = bitrate;
  document.querySelector("#file-size").textContent = size;
}

function syncEncoder() {
  const { quality: label, speeds, defaultSpeed } = encoders[encoder.value];
  document.querySelector("#quality-label").textContent = label;
  preset.replaceChildren(...speeds.map(speed => new Option(speed, speed, false, speed === defaultSpeed)));
  preset.disabled = speeds.length < 2;
}

function setProgress(value) {
  progress.value = value;
  percent.textContent = `${value}%`;
}

function setQueueMessage(message) {
  if (status.textContent !== message) status.textContent = message;
}

function setRunning(running) {
  outputControls.disabled = running;
  addFile.disabled = running;
  startButton.querySelector("use").setAttribute("href", running ? "#i-stop" : "#i-play");
  startButton.querySelector("span").textContent = running ? "Cancelar" : "Iniciar fila";
}

function resetProgress() {
  setProgress(0);
  rows.forEach(({ jobProgress }) => {
    jobProgress.hidden = true;
    jobProgress.textContent = "";
  });
}

function updateRun(now) {
  if (!run) return;

  const elapsed = now - run.startedAt;
  const chunks = rows.map(({ jobProgress }, index) => {
    const start = Math.floor(index / 2) * run.encoder.duration;
    const duration = run.encoder.duration * (0.86 + (index % 2) * 0.14);
    const value = Math.round(Math.max(0, Math.min(1, (elapsed - start) / duration)) * 100);
    jobProgress.hidden = false;
    if (value === 100) {
      const reduction = 26 + index * 6 + Number(run.quality) - 18;
      jobProgress.textContent = `Concluído · −${reduction}%`;
    } else {
      jobProgress.textContent = elapsed < start ? "Na fila" : `${value}%`;
    }
    return value;
  });

  const converted = chunks.filter(value => value === 100).length;
  setProgress(Math.floor(chunks.reduce((sum, value) => sum + value, 0) / fileTotal));
  if (converted === fileTotal) {
    setQueueMessage(`${fileTotal} de ${fileTotal} vídeos · ${run.container} · Simulação concluída`);
    run = null;
    setRunning(false);
    return;
  }

  const secondsLeft = Math.max(1, Math.ceil((Math.ceil(fileTotal / 2) * run.encoder.duration - elapsed) / 1000));
  setQueueMessage(`${converted}/${fileTotal} vídeos · ${run.encoder.fps} FPS · ETA ${secondsLeft} s`);
}

examples.slice(0, fileTotal).forEach(addRow);
selectFile(0);
syncEncoder();

addFile.addEventListener("click", () => {
  addRow(examples[fileTotal], fileTotal);
  fileTotal += 1;
  fileCount.textContent = fileTotal;
  resetProgress();
  selectFile(fileTotal - 1);
  setQueueMessage(`${fileTotal} vídeos prontos`);
  if (fileTotal === examples.length) {
    rows[fileTotal - 1].button.focus();
    addFile.hidden = true;
  }
});

encoder.addEventListener("change", syncEncoder);
quality.addEventListener("input", () => { qualityValue.value = quality.value; });

let queueTimer;
startButton.addEventListener("click", () => {
  if (run) {
    window.clearInterval(queueTimer);
    run = null;
    resetProgress();
    setQueueMessage("Simulação cancelada.");
    setRunning(false);
    return;
  }

  resetProgress();
  run = {
    startedAt: performance.now(),
    encoder: encoders[encoder.value],
    quality: quality.value,
    container: document.querySelector("#container").value,
  };
  setRunning(true);
  updateRun(performance.now());
  queueTimer = window.setInterval(() => {
    updateRun(performance.now());
    if (!run) window.clearInterval(queueTimer);
  }, 150);
});

document.querySelectorAll("[data-install]").forEach(link => {
  link.addEventListener("click", event => {
    event.preventDefault();
    dialog.showModal();
  });
});

function setMenu(open) {
  siteNav.classList.toggle("is-open", open);
  menu.setAttribute("aria-expanded", String(open));
}

menu.addEventListener("click", () => setMenu(menu.getAttribute("aria-expanded") === "false"));
siteNav.addEventListener("click", event => {
  if (event.target.matches("a")) setMenu(false);
});
document.querySelector(".site-header").addEventListener("keydown", event => {
  if (event.key === "Escape" && menu.getAttribute("aria-expanded") === "true") {
    setMenu(false);
    menu.focus();
  }
});
