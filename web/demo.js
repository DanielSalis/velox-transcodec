"use strict";

const examples = [
  { name: "entrevista_01.mov", codec: "ProRes", resolution: "3840×2160", duration: "02:14", fps: "29,97", bitrate: "428 Mb/s", size: "7,2 GB" },
  { name: "campo_aberto.mp4", codec: "HEVC", resolution: "3840×2160", duration: "01:38", fps: "59,94", bitrate: "53 Mb/s", size: "652 MB" },
  { name: "captura_tela.webm", codec: "VP9", resolution: "2560×1440", duration: "00:52", fps: "60", bitrate: "14 Mb/s", size: "91 MB" },
  { name: "cidade_noite.mkv", codec: "AV1", resolution: "1920×1080", duration: "00:46", fps: "24", bitrate: "8 Mb/s", size: "46 MB" },
  { name: "bastidores.mp4", codec: "H.264", resolution: "1920×1080", duration: "01:20", fps: "30", bitrate: "19 Mb/s", size: "190 MB" },
];

const encoders = {
  vaapi: { quality: "Quantizador (CQP)", presets: ["Automático"], defaultPreset: "Automático", fps: 112, milliseconds: 2300 },
  nvenc: { quality: "Qualidade constante", presets: ["P1", "P2", "P3", "P4", "P5", "P6", "P7"], defaultPreset: "P4", fps: 144, milliseconds: 1900 },
  x264: {
    quality: "CRF",
    presets: ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"],
    defaultPreset: "medium",
    fps: 62,
    milliseconds: 2700,
  },
};

function fileIcon() {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  const use = document.createElementNS("http://www.w3.org/2000/svg", "use");
  svg.classList.add("icon");
  svg.setAttribute("aria-hidden", "true");
  use.setAttribute("href", "#i-video");
  svg.append(use);
  return svg;
}

function bindFiles(panel) {
  const rows = panel.querySelector("[data-rows]");
  const count = panel.querySelector("[data-count]");
  const add = panel.querySelector("[data-add]");
  const include = panel.querySelector("[data-include]");
  const details = [panel.querySelector("[data-fps]"), panel.querySelector("[data-bitrate]"), panel.querySelector("[data-size]")];
  const entries = [];
  let selected;
  let locked = false;

  function updateInclusion() {
    if (include) {
      include.textContent = selected.included ? "Excluir da fila" : "Incluir na fila";
      include.disabled = locked;
    }
  }

  function select(entry) {
    selected?.button.setAttribute("aria-pressed", "false");
    selected = entry;
    entry.button.setAttribute("aria-pressed", "true");
    [entry.file.fps, entry.file.bitrate, entry.file.size].forEach((value, index) => {
      details[index].textContent = value;
    });
    updateInclusion();
  }

  function append(file) {
    const row = rows.insertRow();
    const button = document.createElement("button");
    const name = document.createElement("span");
    const state = document.createElement("span");
    const progress = document.createElement("progress");
    const entry = { file, button, state, progress, included: true, finished: false };

    button.className = "file-button";
    button.type = "button";
    button.title = file.name;
    button.setAttribute("aria-pressed", "false");
    button.addEventListener("click", () => select(entry));
    name.textContent = file.name;
    state.className = "file-progress";
    state.textContent = "Pronto";
    progress.className = "file-meter";
    progress.hidden = true;
    progress.max = 100;
    progress.setAttribute("aria-label", "Progresso de " + file.name);
    button.append(fileIcon(), name);
    row.insertCell().append(button, state, progress);
    for (const value of [file.codec, file.resolution, file.duration]) row.insertCell().textContent = value;
    entries.push(entry);
    if (count) count.textContent = entries.length;
    return entry;
  }

  function includedEntries() {
    return include ? entries.filter(entry => entry.included) : entries;
  }

  panel.dataset.files.split(",").forEach(index => append(examples[Number(index)]));
  select(entries[0]);

  include?.addEventListener("click", () => {
    selected.included = !selected.included;
    if (!selected.finished) selected.state.textContent = selected.included ? "Pronto" : "Fora da fila";
    updateInclusion();
    panel.dispatchEvent(new Event("demofileschange", { bubbles: true }));
  });

  add?.addEventListener("click", () => {
    const unused = examples.find(file => !entries.some(entry => entry.file === file));
    if (!unused) return;
    const entry = append(unused);
    select(entry);
    if (entries.length === examples.length) {
      entry.button.focus();
      add.hidden = true;
    }
    panel.dispatchEvent(new Event("demofileschange", { bubbles: true }));
  });

  return {
    includedEntries,
    lock(value) {
      locked = value;
      if (add) add.disabled = locked;
      updateInclusion();
    },
    complete(entry) {
      entry.finished = true;
      if (include) entry.included = false;
      updateInclusion();
    },
  };
}

function bindEncoding(form) {
  const encoder = form.querySelector("[data-encoder]");
  const preset = form.querySelector("[data-preset]");
  const profile = form.querySelector("[data-profile]");
  const quality = form.querySelector("[data-quality]");
  const qualityValue = form.querySelector("[data-quality-value]");
  const qualityLabel = form.querySelector("[data-quality-label]");
  const container = form.querySelector("[data-container]");

  function sync() {
    const settings = encoders[encoder.value];
    qualityLabel.textContent = settings.quality;
    preset.replaceChildren(...settings.presets.map(speed => new Option(speed, speed, false, speed === settings.defaultPreset)));
    preset.disabled = settings.presets.length === 1;
    profile.selectedIndex = 0;
  }

  sync();
  encoder.addEventListener("change", sync);
  quality.addEventListener("input", () => { qualityValue.value = quality.value; });
  return {
    read: () => ({ key: encoder.value, quality: Number(quality.value), container: container.value }),
    lock: value => { form.disabled = value; },
  };
}

function timeLabel(milliseconds) {
  const seconds = Math.max(0, Math.ceil(milliseconds / 1000));
  return String(Math.floor(seconds / 60)).padStart(2, "0") + ":" + String(seconds % 60).padStart(2, "0");
}

function bindQueue(root, files, encoding) {
  const progress = root.querySelector("[data-progress]");
  const percent = root.querySelector("[data-percent]");
  const status = root.querySelector("[data-queue-status]");
  const announcement = root.querySelector("[data-queue-announcement]");
  const start = root.querySelector("[data-start]");
  const buttonIcon = start.querySelector("use");
  const buttonText = start.querySelector("span");
  let run;
  let timer;

  function setProgress(value) {
    progress.value = value;
    percent.textContent = Math.floor(value) + "%";
  }

  function ready() {
    const count = files.includedEntries().length;
    start.disabled = count === 0;
    status.textContent = count === 1 ? "1 vídeo pronto" : count > 0 ? count + " vídeos prontos" : "Inclua um arquivo para iniciar a fila.";
  }

  function finish(cancelled) {
    window.clearInterval(timer);
    const complete = run.jobs.filter(job => job.done).length;
    const total = run.jobs.length;
    if (cancelled) {
      for (const job of run.jobs) {
        if (!job.done) {
          job.entry.state.textContent = "Cancelado";
          job.entry.progress.hidden = true;
        }
      }
    }
    setProgress(100);
    status.textContent = cancelled
      ? complete + " de " + total + " vídeos concluídos · Simulação cancelada"
      : complete + " de " + total + " vídeos · " + run.container + " · Simulação concluída";
    announcement.textContent = status.textContent;
    run = undefined;
    files.lock(false);
    encoding?.lock(false);
    start.classList.replace("secondary", "primary");
    start.disabled = files.includedEntries().length === 0;
    buttonIcon.setAttribute("href", "#i-play");
    buttonText.textContent = "Iniciar fila";
  }

  function update() {
    const elapsed = performance.now() - run.startedAt;
    let weighted = 0;
    let complete = 0;
    for (const job of run.jobs) {
      const value = Math.max(0, Math.min(1, (elapsed - job.start) / job.duration));
      weighted += value * job.weight;
      if (value === 1) complete += 1;
      if (job.done) continue;
      if (value === 1) {
        job.done = true;
        job.entry.state.textContent = "Concluído · −" + job.reduction + "% de tamanho";
        job.entry.progress.value = 100;
        files.complete(job.entry);
      } else if (elapsed >= job.start) {
        const itemPercent = Math.floor(value * 100);
        job.entry.state.textContent = "Codificando · " + itemPercent + "% · " + run.fps + " FPS · ETA " + timeLabel(job.start + job.duration - elapsed);
        job.entry.progress.hidden = false;
        job.entry.progress.value = itemPercent;
      }
    }
    setProgress(100 * weighted / run.weight);
    if (complete === run.jobs.length) {
      finish(false);
    } else {
      status.textContent = complete + " de " + run.jobs.length + " vídeos · ETA " + timeLabel(run.endsAt - elapsed);
    }
  }

  start.addEventListener("click", () => {
    if (run) {
      finish(true);
      return;
    }
    const entries = files.includedEntries();
    if (!entries.length) return;
    const settings = encoding?.read() ?? { key: "x264", quality: 22, container: "MP4" };
    const { fps, milliseconds } = encoders[settings.key];
    const slots = settings.key === "x264" ? [0, 0] : [0];
    const jobs = entries.map((entry, index) => {
      const slot = slots.indexOf(Math.min(...slots));
      const start = slots[slot];
      const [minutes, seconds] = entry.file.duration.split(":").map(Number);
      const weight = minutes * 60 + seconds;
      const duration = milliseconds * (0.7 + weight / 360);
      slots[slot] += duration;
      entry.finished = false;
      entry.state.textContent = "Na fila";
      entry.progress.hidden = true;
      entry.progress.value = 0;
      return { entry, start, duration, weight, reduction: 26 + index * 6 + settings.quality - 18, done: false };
    });

    run = {
      jobs, fps, container: settings.container, startedAt: performance.now(),
      endsAt: Math.max(...slots), weight: jobs.reduce((sum, job) => sum + job.weight, 0),
    };
    files.lock(true);
    encoding?.lock(true);
    start.classList.replace("primary", "secondary");
    buttonIcon.setAttribute("href", "#i-stop");
    buttonText.textContent = "Cancelar";
    announcement.textContent = "Simulação iniciada.";
    setProgress(0);
    timer = window.setInterval(update, 150);
    update();
  });

  root.addEventListener("demofileschange", ready);
  ready();
}

for (const root of document.querySelectorAll("[data-demo]")) {
  const filesPanel = root.querySelector("[data-files]");
  const encodingForm = root.querySelector("[data-encoding]");
  const files = filesPanel && bindFiles(filesPanel);
  const encoding = encodingForm && bindEncoding(encodingForm);
  if (root.querySelector("[data-queue]")) bindQueue(root, files, encoding);
}

function legacyCopy(text, button) {
  const field = document.createElement("textarea");
  field.className = "sr-only";
  field.value = text;
  field.readOnly = true;
  document.body.append(field);
  field.select();
  try {
    return document.execCommand("copy");
  } finally {
    field.remove();
    button.focus({ preventScroll: true });
  }
}

for (const button of document.querySelectorAll("[data-copy]")) {
  const code = document.getElementById(button.dataset.copy);
  const status = button.parentElement.querySelector("[data-copy-status]");
  const label = button.querySelector("span");
  let timer;
  button.addEventListener("click", async () => {
    window.clearTimeout(timer);
    let copied = false;
    try {
      await navigator.clipboard.writeText(code.textContent);
      copied = true;
    } catch {
      try {
        copied = legacyCopy(code.textContent, button);
      } catch {
        copied = false;
      }
    }
    if (!copied) {
      code.focus({ preventScroll: true });
      window.getSelection()?.selectAllChildren(code);
    }
    status.textContent = copied ? "Comando copiado." : "Comando selecionado. Use o atalho de copiar.";
    label.textContent = copied ? "Copiado" : "Selecionado";
    timer = window.setTimeout(() => { label.textContent = "Copiar"; status.textContent = ""; }, 2200);
  });
}

const menu = document.querySelector(".menu-toggle");
const siteNav = document.querySelector("#site-nav");

function setMenu(open) {
  siteNav.classList.toggle("is-open", open);
  menu.setAttribute("aria-expanded", String(open));
}

menu.addEventListener("click", () => setMenu(menu.getAttribute("aria-expanded") === "false"));
siteNav.addEventListener("click", event => {
  if (event.target.closest("a")) setMenu(false);
});
document.querySelector(".site-header").addEventListener("keydown", event => {
  if (event.key === "Escape" && menu.getAttribute("aria-expanded") === "true") {
    setMenu(false);
    menu.focus();
  }
});
