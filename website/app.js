(() => {
  'use strict';

  /* ========================================================================
     Authentic FileWheel Accent Presets (from src/appearance.rs)
     ======================================================================== */
  const ACCENTS = [
    { name: 'Orange', hex: '#FC9259' },
    { name: 'Coral',  hex: '#F36F81' },
    { name: 'Gold',   hex: '#EBB53B' },
    { name: 'Mint',   hex: '#46BE97' },
    { name: 'Teal',   hex: '#2DB8C4' },
    { name: 'Blue',   hex: '#579DF6' },
    { name: 'Violet', hex: '#A485F7' },
    { name: 'Pink',   hex: '#DB7AD1' },
  ];

  const SPEEDS = [5000, 8000, 12000];

  /* ========================================================================
     Authentic Sample Files & Operations (from README.md / src/engine.rs)
     ======================================================================== */
  const SAMPLE_FILES = {
    image: {
      name: 'sunset-photo.png',
      stem: 'sunset-photo',
      family: 'Images',
      engine: 'ImageMagick 7',
      convert: [
        { label: 'JPG',  icon: '🖼️', ext: 'jpg' },
        { label: 'WebP', icon: '⚡', ext: 'webp' },
        { label: 'AVIF', icon: '✨', ext: 'avif' },
        { label: 'HEIC', icon: '📱', ext: 'heic' },
        { label: 'TIFF', icon: '🎞️', ext: 'tiff' },
        { label: 'BMP',  icon: '🧩', ext: 'bmp' },
        { label: 'PDF',  icon: '📄', ext: 'pdf' },
        { label: 'PNG',  icon: '🎨', ext: 'png' },
      ],
      tools: [
        { label: 'Resize 1280', icon: '📐', suffix: '_1280', ext: 'png' },
        { label: 'Compress',    icon: '🗜️', suffix: '_compressed', ext: 'jpg' },
        { label: 'Strip Meta',  icon: '🧹', suffix: '_clean', ext: 'png' },
        { label: 'Rotate 90°',  icon: '🔄', suffix: '_rotated', ext: 'png' },
        { label: 'Grayscale',   icon: '🌗', suffix: '_gray', ext: 'png' },
        { label: 'ZIP',         icon: '📦', suffix: '', ext: 'zip' },
      ],
    },
    video: {
      name: 'demo-capture.mov',
      stem: 'demo-capture',
      family: 'Video & GIF',
      engine: 'FFmpeg',
      convert: [
        { label: 'MP4',  icon: '🎬', ext: 'mp4' },
        { label: 'WebM', icon: '🌐', ext: 'webm' },
        { label: 'MKV',  icon: '🎞️', ext: 'mkv' },
        { label: 'GIF',  icon: '✨', ext: 'gif' },
        { label: 'AVI',  icon: '📼', ext: 'avi' },
        { label: 'MP3',  icon: '🎵', ext: 'mp3' },
      ],
      tools: [
        { label: 'Compress',   icon: '🗜️', suffix: '_compressed', ext: 'mp4' },
        { label: 'Strip Meta', icon: '🧹', suffix: '_clean', ext: 'mov' },
        { label: 'Mute',       icon: '🔇', suffix: '_muted', ext: 'mov' },
        { label: 'First 10s',  icon: '⏱️', suffix: '_10s', ext: 'mov' },
        { label: 'Save Frame', icon: '📸', suffix: '_frame', ext: 'png' },
        { label: 'SHA-256',    icon: '🔑', actionText: 'SHA-256 copied to clipboard' },
      ],
    },
    audio: {
      name: 'master-track.flac',
      stem: 'master-track',
      family: 'Audio',
      engine: 'FFmpeg',
      convert: [
        { label: 'MP3',  icon: '🎵', ext: 'mp3' },
        { label: 'M4A',  icon: '🎧', ext: 'm4a' },
        { label: 'WAV',  icon: '🌊', ext: 'wav' },
        { label: 'Opus', icon: '⚡', ext: 'opus' },
        { label: 'OGG',  icon: '🔊', ext: 'ogg' },
        { label: 'AIFF', icon: '🎹', ext: 'aiff' },
      ],
      tools: [
        { label: 'Normalize',  icon: '📊', suffix: '_norm', ext: 'flac' },
        { label: 'First 30s',  icon: '⏱️', suffix: '_30s', ext: 'flac' },
        { label: 'Mono',       icon: '🔈', suffix: '_mono', ext: 'flac' },
        { label: 'Compress',   icon: '🗜️', suffix: '_small', ext: 'mp3' },
        { label: 'Strip Meta', icon: '🧹', suffix: '_clean', ext: 'flac' },
      ],
    },
    pdf: {
      name: 'quarterly-brief.pdf',
      stem: 'quarterly-brief',
      family: 'PDF',
      engine: 'LibreOffice · Poppler · QPDF',
      convert: [
        { label: 'DOCX', icon: '📝', ext: 'docx' },
        { label: 'PPTX', icon: '📊', ext: 'pptx' },
        { label: 'XLSX', icon: '📈', ext: 'xlsx' },
        { label: 'PNG',  icon: '🖼️', suffix: '_pages', ext: 'png' },
        { label: 'JPG',  icon: '📷', suffix: '_pages', ext: 'jpg' },
        { label: 'EPUB', icon: '📖', ext: 'epub' },
        { label: 'MD',   icon: '📑', ext: 'md' },
        { label: 'TXT',  icon: '📃', ext: 'txt' },
      ],
      tools: [
        { label: 'Strip Meta', icon: '🧹', suffix: '_clean', ext: 'pdf' },
        { label: 'ZIP',        icon: '📦', suffix: '', ext: 'zip' },
        { label: 'SHA-256',    icon: '🔑', actionText: 'SHA-256 copied to clipboard' },
      ],
    },
    doc: {
      name: 'release-notes.docx',
      stem: 'release-notes',
      family: 'Documents',
      engine: 'LibreOffice · Python 3',
      convert: [
        { label: 'PDF',  icon: '📄', ext: 'pdf' },
        { label: 'MD',   icon: '📑', ext: 'md' },
        { label: 'TXT',  icon: '📃', ext: 'txt' },
        { label: 'EPUB', icon: '📖', ext: 'epub' },
        { label: 'HTML', icon: '🌐', ext: 'html' },
      ],
      tools: [
        { label: 'Copy Text',  icon: '📋', actionText: 'Document text copied to clipboard' },
        { label: 'Word Count', icon: '🔢', actionText: 'Copied word count: 1,428 words' },
        { label: 'ZIP',        icon: '📦', suffix: '', ext: 'zip' },
        { label: 'SHA-256',    icon: '🔑', actionText: 'SHA-256 copied to clipboard' },
      ],
    },
    archive: {
      name: 'project-assets.rar',
      stem: 'project-assets',
      family: 'Archives',
      engine: 'libarchive / tar',
      convert: [
        { label: 'ZIP',    icon: '📦', ext: 'zip' },
        { label: 'TAR',    icon: '🗂️', ext: 'tar' },
        { label: 'TAR.GZ', icon: '🗜️', ext: 'tar.gz' },
      ],
      tools: [
        { label: 'Repack ZIP', icon: '♻️', suffix: '_repack', ext: 'zip' },
        { label: 'SHA-256',    icon: '🔑', actionText: 'SHA-256 copied to clipboard' },
      ],
    },
  };

  const COMMON_ACTIONS = [
    { label: 'Copy Path', icon: '📎', actionText: 'Copied full path to clipboard' },
    { label: 'Reveal',    icon: '📂', actionText: 'Revealed file in system file manager' },
    { label: 'Open',      icon: '🚀', actionText: 'Opened with default desktop app' },
    { label: 'Preview',   icon: '👁️', actionText: 'Opened inside FileWheel preview' },
    { label: 'Trash',     icon: '🗑️', actionText: 'Confirmation prompt → Move to desktop Trash' },
  ];

  /* ========================================================================
     State
     ======================================================================== */
  const slides = Array.from(document.querySelectorAll('.slide'));
  const totalSlides = slides.length;

  let currentSlide = 0;
  let autoPlay = true;
  let speedIndex = 1; // 8000ms
  let slideElapsed = 0;
  let lastFrameTime = null;
  let rafId = null;

  let currentFileId = 'image';
  let currentWheelMode = 'convert';
  let selectedWedgeIdx = -1;
  const outputCounts = new Map();

  let hoverGlow = true;
  let hoverSound = true;
  let themeMode = 'dark';
  let currentAccent = ACCENTS[0].hex;
  let lastSoundTime = 0;
  const hoverAudio = new Audio('assets/hover.wav');
  hoverAudio.volume = 0.28;

  /* ========================================================================
     DOM References
     ======================================================================== */
  const autoSlideFill = document.getElementById('autoSlideFill');
  const topbarSlideLabel = document.getElementById('topbarSlideLabel');
  const slideDotsEl = document.getElementById('slideDots');
  const prevSlideBtn = document.getElementById('prevSlideBtn');
  const nextSlideBtn = document.getElementById('nextSlideBtn');
  const autoPlayToggleBtn = document.getElementById('autoPlayToggleBtn');
  const autoPlayLabel = document.getElementById('autoPlayLabel');
  const speedCycleBtn = document.getElementById('speedCycleBtn');
  const speedLabel = document.getElementById('speedLabel');
  const brandHomeBtn = document.getElementById('brandHomeBtn');
  const themeToggleBtn = document.getElementById('themeToggle');
  const soundToggleBtn = document.getElementById('soundToggleBtn');
  const fullscreenBtn = document.getElementById('fullscreenBtn');

  // Overview & Lightbox
  const overviewModal = document.getElementById('overviewModal');
  const overviewGrid = document.getElementById('overviewGrid');
  const overviewToggleBtn = document.getElementById('overviewToggleBtn');
  const overviewCloseBtn = document.getElementById('overviewCloseBtn');
  const overviewBackdrop = document.getElementById('overviewBackdrop');

  const lightboxModal = document.getElementById('lightboxModal');
  const lightboxImg = document.getElementById('lightboxImg');
  const lightboxCaption = document.getElementById('lightboxCaption');
  const lightboxCloseBtn = document.getElementById('lightboxCloseBtn');
  const lightboxBackdrop = document.getElementById('lightboxBackdrop');

  // Simulator
  const radialWheelSvg = document.getElementById('radialWheelSvg');
  const simFamilyBadge = document.getElementById('simFamilyBadge');
  const simEnginePill = document.getElementById('simEnginePill');
  const wcModeLabel = document.getElementById('wcModeLabel');
  const wcTitle = document.getElementById('wcTitle');
  const wcSub = document.getElementById('wcSub');
  const simStatusDot = document.getElementById('simStatusDot');
  const simStatusTitle = document.getElementById('simStatusTitle');
  const simProgressFill = document.getElementById('simProgressFill');
  const simOutputPath = document.getElementById('simOutputPath');
  const wheelDropZone = document.getElementById('wheelDropZone');
  const floatingTargetDemo = document.getElementById('floatingTargetDemo');

  // Appearance Studio
  const accentSwatchGrid = document.getElementById('accentSwatchGrid');
  const customColorPicker = document.getElementById('customColorPicker');
  const customHexInput = document.getElementById('customHexInput');
  const contrastBadge = document.getElementById('contrastBadge');
  const hoverGlowCheck = document.getElementById('hoverGlowCheck');
  const hoverSoundCheck = document.getElementById('hoverSoundCheck');
  const testEffectsBtn = document.getElementById('testEffectsBtn');
  const testEffectsCaption = document.getElementById('testEffectsCaption');
  const restoreAccentBtn = document.getElementById('restoreAccentBtn');

  /* ========================================================================
     Color Contrast & Dynamic Accent Engine (matches src/appearance.rs)
     ======================================================================== */
  function parseHex(hex) {
    const clean = hex.trim().replace(/^#/, '');
    if (!/^[0-9a-fA-F]{6}$/.test(clean)) return null;
    return {
      r: parseInt(clean.slice(0, 2), 16),
      g: parseInt(clean.slice(2, 4), 16),
      b: parseInt(clean.slice(4, 6), 16),
      hex: '#' + clean.toUpperCase(),
    };
  }

  function luminance({ r, g, b }) {
    const lin = (v) => {
      const n = v / 255;
      return n <= 0.04045 ? n / 12.92 : Math.pow((n + 0.055) / 1.055, 2.4);
    };
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
  }

  function contrastRatio(c1, c2) {
    const l1 = luminance(c1);
    const l2 = luminance(c2);
    return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
  }

  function applyAccent(hex) {
    const parsed = parseHex(hex);
    if (!parsed) return false;
    currentAccent = parsed.hex;

    const darkInk = { r: 16, g: 20, b: 27 };
    const lightInk = { r: 255, g: 255, b: 255 };
    const darkRatio = contrastRatio(parsed, darkInk);
    const lightRatio = contrastRatio(parsed, lightInk);
    const inkHex = darkRatio >= lightRatio ? '#10141B' : '#FFFFFF';
    const bestRatio = Math.max(darkRatio, lightRatio).toFixed(1);

    const root = document.documentElement.style;
    root.setProperty('--accent', parsed.hex);
    root.setProperty('--accent-rgb', `${parsed.r}, ${parsed.g}, ${parsed.b}`);
    root.setProperty('--accent-ink', inkHex);
    root.setProperty('--accent-soft', `rgba(${parsed.r}, ${parsed.g}, ${parsed.b}, 0.16)`);
    root.setProperty('--accent-border', `rgba(${parsed.r}, ${parsed.g}, ${parsed.b}, 0.44)`);
    root.setProperty('--accent-glow', `rgba(${parsed.r}, ${parsed.g}, ${parsed.b}, 0.36)`);

    customColorPicker.value = parsed.hex;
    customHexInput.value = parsed.hex;
    contrastBadge.textContent = `Ink: ${inkHex === '#10141B' ? 'Dark' : 'Light'} (${bestRatio}:1)`;

    document.querySelectorAll('.swatch-btn').forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.hex === parsed.hex);
    });
    return true;
  }

  function applyTheme(mode) {
    themeMode = mode;
    let resolved = mode;
    if (mode === 'system') {
      resolved = window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    }
    document.documentElement.setAttribute('data-theme', resolved);
    document.querySelectorAll('[data-set-theme]').forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.setTheme === mode);
    });
  }

  function playHoverCue() {
    if (!hoverSound) return;
    const now = performance.now();
    if (now - lastSoundTime < 90) return;
    lastSoundTime = now;
    try {
      hoverAudio.currentTime = 0;
      hoverAudio.play().catch(() => {});
    } catch (_) {}
  }

  /* ========================================================================
     Slide Navigation & Auto-Slide Engine
     ======================================================================== */
  function buildDotsAndOverview() {
    slideDotsEl.innerHTML = '';
    overviewGrid.innerHTML = '';
    slides.forEach((slide, idx) => {
      const title = slide.dataset.title || `Slide ${idx + 1}`;

      const dot = document.createElement('button');
      dot.type = 'button';
      dot.className = 'slide-dot' + (idx === 0 ? ' active' : '');
      dot.title = `${String(idx + 1).padStart(2, '0')} · ${title}`;
      dot.setAttribute('aria-label', `Go to slide ${idx + 1}: ${title}`);
      dot.addEventListener('click', () => goToSlide(idx));
      slideDotsEl.appendChild(dot);

      const card = document.createElement('button');
      card.type = 'button';
      card.className = 'overview-card' + (idx === 0 ? ' active' : '');
      card.innerHTML = `<span>${String(idx + 1).padStart(2, '0')} / ${String(totalSlides).padStart(2, '0')}</span><strong>${title}</strong>`;
      card.addEventListener('click', () => {
        goToSlide(idx);
        closeOverview();
      });
      overviewGrid.appendChild(card);
    });
  }

  function goToSlide(index, resetTimer = true) {
    const target = ((index % totalSlides) + totalSlides) % totalSlides;
    currentSlide = target;

    slides.forEach((slide, i) => {
      slide.classList.remove('active', 'prev');
      if (i === currentSlide) {
        slide.classList.add('active');
        slide.scrollTop = 0;
      } else if (i < currentSlide) {
        slide.classList.add('prev');
      }
    });

    const title = slides[currentSlide].dataset.title || '';
    topbarSlideLabel.textContent = `${String(currentSlide + 1).padStart(2, '0')} / ${String(totalSlides).padStart(2, '0')} · ${title}`;

    slideDotsEl.querySelectorAll('.slide-dot').forEach((d, i) => {
      d.classList.toggle('active', i === currentSlide);
    });
    overviewGrid.querySelectorAll('.overview-card').forEach((c, i) => {
      c.classList.toggle('active', i === currentSlide);
    });

    if (resetTimer) {
      slideElapsed = 0;
      autoSlideFill.style.width = '0%';
    }
    history.replaceState(null, '', `#slide-${currentSlide + 1}`);
  }

  function setAutoPlay(enabled) {
    autoPlay = enabled;
    autoPlayToggleBtn.setAttribute('aria-pressed', String(autoPlay));
    autoPlayLabel.textContent = autoPlay ? 'Auto' : 'Paused';
    if (!autoPlay) {
      autoSlideFill.style.width = `${Math.min(100, (slideElapsed / SPEEDS[speedIndex]) * 100)}%`;
    } else {
      lastFrameTime = performance.now();
    }
  }

  function tickAutoSlide(now) {
    if (lastFrameTime === null) lastFrameTime = now;
    const dt = now - lastFrameTime;
    lastFrameTime = now;

    const modalOpen = !overviewModal.hidden || !lightboxModal.hidden;
    if (autoPlay && !modalOpen && !document.hidden) {
      slideElapsed += dt;
      const duration = SPEEDS[speedIndex];
      const pct = Math.min(100, (slideElapsed / duration) * 100);
      autoSlideFill.style.width = `${pct}%`;

      if (slideElapsed >= duration) {
        goToSlide(currentSlide + 1, true);
      }
    }
    rafId = requestAnimationFrame(tickAutoSlide);
  }

  /* ========================================================================
     Interactive SVG Radial Wheel Generator
     ======================================================================== */
  function polarToCartesian(cx, cy, r, angleDeg) {
    const rad = ((angleDeg - 90) * Math.PI) / 180;
    return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) };
  }

  function buildWedgePath(cx, cy, rIn, rOut, startDeg, endDeg) {
    const gapDeg = 3.2;
    const a1 = startDeg + gapDeg / 2;
    const a2 = endDeg - gapDeg / 2;

    const p1 = polarToCartesian(cx, cy, rOut, a1);
    const p2 = polarToCartesian(cx, cy, rOut, a2);
    const p3 = polarToCartesian(cx, cy, rIn, a2);
    const p4 = polarToCartesian(cx, cy, rIn, a1);
    const largeArc = a2 - a1 > 180 ? 1 : 0;

    return [
      `M ${p1.x.toFixed(2)} ${p1.y.toFixed(2)}`,
      `A ${rOut} ${rOut} 0 ${largeArc} 1 ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`,
      `L ${p3.x.toFixed(2)} ${p3.y.toFixed(2)}`,
      `A ${rIn} ${rIn} 0 ${largeArc} 0 ${p4.x.toFixed(2)} ${p4.y.toFixed(2)}`,
      'Z',
    ].join(' ');
  }

  function getActiveOperations() {
    const file = SAMPLE_FILES[currentFileId];
    if (currentWheelMode === 'actions') return COMMON_ACTIONS;
    return file[currentWheelMode] || [];
  }

  function renderRadialWheel() {
    const file = SAMPLE_FILES[currentFileId];
    const ops = getActiveOperations();
    const cx = 210;
    const cy = 210;
    const rIn = 78;
    const rOut = 192;
    const step = 360 / ops.length;

    simFamilyBadge.textContent = file.family;
    simEnginePill.textContent = `Engine: ${file.engine}`;
    wcModeLabel.textContent = currentWheelMode.toUpperCase();
    wcTitle.textContent = file.name;
    wcSub.textContent = `${ops.length} operations`;

    radialWheelSvg.innerHTML = '';

    ops.forEach((op, idx) => {
      const startDeg = idx * step - step / 2;
      const endDeg = startDeg + step;
      const midDeg = idx * step;

      const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
      g.setAttribute('class', 'wheel-wedge-group' + (idx === selectedWedgeIdx ? ' selected' : ''));
      g.setAttribute('tabindex', '0');
      g.setAttribute('role', 'button');
      g.setAttribute('aria-label', `${currentWheelMode}: ${op.label}`);

      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      path.setAttribute('class', 'wheel-wedge-path');
      path.setAttribute('d', buildWedgePath(cx, cy, rIn, rOut, startDeg, endDeg));
      path.setAttribute('stroke-linejoin', 'round');

      const iconPos = polarToCartesian(cx, cy, (rIn + rOut) / 2 - 11, midDeg);
      const labelPos = polarToCartesian(cx, cy, (rIn + rOut) / 2 + 13, midDeg);

      const iconEl = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      iconEl.setAttribute('class', 'wheel-wedge-icon');
      iconEl.setAttribute('x', iconPos.x.toFixed(1));
      iconEl.setAttribute('y', iconPos.y.toFixed(1));
      iconEl.textContent = op.icon;

      const labelEl = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      labelEl.setAttribute('class', 'wheel-wedge-label');
      labelEl.setAttribute('x', labelPos.x.toFixed(1));
      labelEl.setAttribute('y', labelPos.y.toFixed(1));
      labelEl.textContent = op.label;

      g.appendChild(path);
      g.appendChild(iconEl);
      g.appendChild(labelEl);

      g.addEventListener('mouseenter', () => {
        selectedWedgeIdx = idx;
        updateWedgeSelection();
        playHoverCue();
        wcSub.textContent = op.label;
      });
      g.addEventListener('mouseleave', () => {
        wcSub.textContent = `${ops.length} operations`;
      });
      g.addEventListener('click', () => {
        setAutoPlay(false);
        runSimulatedJob(op);
      });
      g.addEventListener('dragover', (e) => {
        e.preventDefault();
        g.classList.add('drag-hover');
      });
      g.addEventListener('dragleave', () => {
        g.classList.remove('drag-hover');
      });
      g.addEventListener('drop', (e) => {
        e.preventDefault();
        g.classList.remove('drag-hover');
        const draggedId = e.dataTransfer.getData('text/plain');
        if (draggedId && SAMPLE_FILES[draggedId]) {
          selectSampleFile(draggedId);
        }
        setAutoPlay(false);
        runSimulatedJob(op);
      });

      radialWheelSvg.appendChild(g);
    });
  }

  function updateWedgeSelection() {
    radialWheelSvg.querySelectorAll('.wheel-wedge-group').forEach((g, i) => {
      g.classList.toggle('selected', i === selectedWedgeIdx);
    });
  }

  let jobTimer = null;
  function runSimulatedJob(op) {
    const file = SAMPLE_FILES[currentFileId];
    clearTimeout(jobTimer);

    simStatusDot.classList.remove('done');
    simStatusTitle.textContent = `Running ${op.label} on ${file.name}…`;
    simProgressFill.style.width = '28%';

    if (op.actionText) {
      jobTimer = setTimeout(() => {
        simProgressFill.style.width = '100%';
        simStatusDot.classList.add('done');
        simStatusTitle.textContent = `Done · ${op.label}`;
        simOutputPath.textContent = op.actionText;
      }, 260);
      return;
    }

    const baseKey = `${file.stem}${op.suffix || ''}.${op.ext}`;
    const count = (outputCounts.get(baseKey) || 0) + 1;
    outputCounts.set(baseKey, count);
    const finalName = count === 1
      ? baseKey
      : `${file.stem}${op.suffix || ''}_${count}.${op.ext}`;

    setTimeout(() => { simProgressFill.style.width = '76%'; }, 140);
    jobTimer = setTimeout(() => {
      simProgressFill.style.width = '100%';
      simStatusDot.classList.add('done');
      simStatusTitle.textContent = `Saved beside original (${file.engine})`;
      simOutputPath.textContent = `~/Files/${finalName}`;
    }, 460);
  }

  function selectSampleFile(fileId) {
    if (!SAMPLE_FILES[fileId]) return;
    currentFileId = fileId;
    selectedWedgeIdx = -1;
    document.querySelectorAll('.sample-file').forEach((el) => {
      el.classList.toggle('active', el.dataset.fileId === fileId);
    });
    renderRadialWheel();
  }

  function setWheelMode(mode) {
    currentWheelMode = mode;
    selectedWedgeIdx = -1;
    document.querySelectorAll('[data-wheel-mode]').forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.wheelMode === mode);
    });
    renderRadialWheel();
  }

  /* ========================================================================
     Overview & Lightbox Helpers
     ======================================================================== */
  function openOverview() {
    overviewModal.hidden = false;
  }
  function closeOverview() {
    overviewModal.hidden = true;
  }
  function openLightbox(src, caption) {
    lightboxImg.src = src;
    lightboxImg.alt = caption || 'FileWheel screenshot';
    lightboxCaption.textContent = caption || '';
    lightboxModal.hidden = false;
  }
  function closeLightbox() {
    lightboxModal.hidden = true;
  }

  /* ========================================================================
     Appearance Studio Swatches & Controls
     ======================================================================== */
  function buildAccentSwatches() {
    accentSwatchGrid.innerHTML = '';
    ACCENTS.forEach((accent) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'swatch-btn' + (accent.hex === currentAccent ? ' active' : '');
      btn.dataset.hex = accent.hex;
      btn.innerHTML = `<span class="swatch-dot" style="background:${accent.hex}"></span><span>${accent.name}</span>`;
      btn.addEventListener('click', () => {
        setAutoPlay(false);
        applyAccent(accent.hex);
        playHoverCue();
      });
      accentSwatchGrid.appendChild(btn);
    });
  }

  /* ========================================================================
     Event Wiring
     ======================================================================== */
  function bindEvents() {
    prevSlideBtn.addEventListener('click', () => goToSlide(currentSlide - 1));
    nextSlideBtn.addEventListener('click', () => goToSlide(currentSlide + 1));
    brandHomeBtn.addEventListener('click', () => goToSlide(0));

    autoPlayToggleBtn.addEventListener('click', () => setAutoPlay(!autoPlay));
    speedCycleBtn.addEventListener('click', () => {
      speedIndex = (speedIndex + 1) % SPEEDS.length;
      speedLabel.textContent = `${SPEEDS[speedIndex] / 1000}s`;
      slideElapsed = 0;
    });

    themeToggleBtn.addEventListener('click', () => {
      const current = document.documentElement.getAttribute('data-theme');
      applyTheme(current === 'dark' ? 'light' : 'dark');
    });

    soundToggleBtn.addEventListener('click', () => {
      hoverSound = !hoverSound;
      soundToggleBtn.setAttribute('aria-pressed', String(hoverSound));
      hoverSoundCheck.checked = hoverSound;
      if (hoverSound) playHoverCue();
    });

    fullscreenBtn.addEventListener('click', () => {
      if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen?.().catch(() => {});
      } else {
        document.exitFullscreen?.().catch(() => {});
      }
    });

    // Quick slide jump buttons
    document.querySelectorAll('[data-go-slide]').forEach((btn) => {
      btn.addEventListener('click', () => {
        goToSlide(Number(btn.dataset.goSlide));
      });
    });

    // Overview & Lightbox
    overviewToggleBtn.addEventListener('click', () => {
      overviewModal.hidden ? openOverview() : closeOverview();
    });
    overviewCloseBtn.addEventListener('click', closeOverview);
    overviewBackdrop.addEventListener('click', closeOverview);

    document.querySelectorAll('[data-lightbox]').forEach((btn) => {
      btn.addEventListener('click', () => {
        openLightbox(btn.dataset.lightbox, btn.dataset.caption);
      });
    });
    lightboxCloseBtn.addEventListener('click', closeLightbox);
    lightboxBackdrop.addEventListener('click', closeLightbox);

    // Simulator mode & sample files
    document.querySelectorAll('[data-wheel-mode]').forEach((btn) => {
      btn.addEventListener('click', () => {
        setAutoPlay(false);
        setWheelMode(btn.dataset.wheelMode);
      });
    });

    document.querySelectorAll('.sample-file').forEach((btn) => {
      btn.addEventListener('click', () => {
        setAutoPlay(false);
        selectSampleFile(btn.dataset.fileId);
        playHoverCue();
      });
      btn.addEventListener('dragstart', (e) => {
        e.dataTransfer.setData('text/plain', btn.dataset.fileId);
        selectSampleFile(btn.dataset.fileId);
      });
    });

    // Drop onto floating target demo
    [floatingTargetDemo, wheelDropZone].forEach((zone) => {
      zone.addEventListener('dragover', (e) => {
        e.preventDefault();
        zone.classList.add('drag-over');
      });
      zone.addEventListener('dragleave', () => zone.classList.remove('drag-over'));
      zone.addEventListener('drop', (e) => {
        e.preventDefault();
        zone.classList.remove('drag-over');
        const id = e.dataTransfer.getData('text/plain');
        if (id && SAMPLE_FILES[id]) {
          selectSampleFile(id);
          simStatusTitle.textContent = `Staged ${SAMPLE_FILES[id].name} — pick a wedge`;
        }
      });
    });

    // Format filters (Slide 4)
    document.querySelectorAll('[data-fmt-filter]').forEach((btn) => {
      btn.addEventListener('click', () => {
        setAutoPlay(false);
        const cat = btn.dataset.fmtFilter;
        document.querySelectorAll('[data-fmt-filter]').forEach((b) => b.classList.toggle('active', b === btn));
        document.querySelectorAll('.format-card').forEach((card) => {
          card.classList.toggle('hidden', cat !== 'all' && card.dataset.cat !== cat);
        });
      });
    });

    // Appearance Studio (Slide 5)
    document.querySelectorAll('[data-set-theme]').forEach((btn) => {
      btn.addEventListener('click', () => {
        setAutoPlay(false);
        applyTheme(btn.dataset.setTheme);
      });
    });

    customColorPicker.addEventListener('input', (e) => {
      setAutoPlay(false);
      applyAccent(e.target.value);
    });

    customHexInput.addEventListener('input', (e) => {
      setAutoPlay(false);
      applyAccent(e.target.value);
    });

    hoverGlowCheck.addEventListener('change', (e) => {
      hoverGlow = e.target.checked;
      document.body.classList.toggle('glow-enabled', hoverGlow);
    });

    hoverSoundCheck.addEventListener('change', (e) => {
      hoverSound = e.target.checked;
      soundToggleBtn.setAttribute('aria-pressed', String(hoverSound));
    });

    testEffectsBtn.addEventListener('click', () => {
      setAutoPlay(false);
      if (!hoverGlow && !hoverSound) {
        testEffectsCaption.textContent = 'Enable Accent glow or Hover sound above to test.';
        return;
      }
      const tested = [];
      if (hoverGlow) {
        tested.push('glow');
        testEffectsBtn.classList.add('effect-preview-active');
        setTimeout(() => testEffectsBtn.classList.remove('effect-preview-active'), 550);
      }
      if (hoverSound) {
        tested.push('sound');
        playHoverCue();
      }
      testEffectsCaption.textContent = `Testing ${tested.join(' + ')} preview!`;
    });

    restoreAccentBtn.addEventListener('click', () => {
      applyTheme('dark');
      applyAccent(ACCENTS[0].hex);
      hoverGlow = true;
      hoverSound = true;
      hoverGlowCheck.checked = true;
      hoverSoundCheck.checked = true;
      soundToggleBtn.setAttribute('aria-pressed', 'true');
      document.body.classList.add('glow-enabled');
      testEffectsCaption.textContent = 'Restored default Dark + Orange theme.';
    });

    // Install Tabs (Slide 8)
    document.querySelectorAll('[data-install-tab]').forEach((btn) => {
      btn.addEventListener('click', () => {
        setAutoPlay(false);
        const tab = btn.dataset.installTab;
        document.querySelectorAll('[data-install-tab]').forEach((b) => b.classList.toggle('active', b === btn));
        document.querySelectorAll('[data-install-panel]').forEach((p) => {
          p.classList.toggle('active', p.dataset.installPanel === tab);
        });
      });
    });

    // Copy buttons
    document.querySelectorAll('[data-copy-target]').forEach((btn) => {
      btn.addEventListener('click', async () => {
        setAutoPlay(false);
        const targetEl = document.getElementById(btn.dataset.copyTarget);
        if (!targetEl) return;
        try {
          await navigator.clipboard.writeText(targetEl.textContent.trim());
          const orig = btn.textContent;
          btn.textContent = 'Copied ✓';
          setTimeout(() => { btn.textContent = orig; }, 1600);
        } catch (_) {
          btn.textContent = 'Select text';
        }
      });
    });

    // Keyboard shortcuts
    window.addEventListener('keydown', (e) => {
      if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

      if (e.key === 'Escape') {
        if (!lightboxModal.hidden) return closeLightbox();
        if (!overviewModal.hidden) return closeOverview();
        return;
      }

      // On Live Wheel slide, support authentic Shift/Ctrl/Alt mode switching
      if (currentSlide === 1 && !e.repeat) {
        if (e.key === 'Shift') { setWheelMode('convert'); return; }
        if (e.key === 'Control') { setWheelMode('tools'); return; }
        if (e.key === 'Alt') { e.preventDefault(); setWheelMode('actions'); return; }
      }

      if (e.key === 'ArrowRight' || e.key === 'PageDown') {
        e.preventDefault();
        goToSlide(currentSlide + 1);
      } else if (e.key === 'ArrowLeft' || e.key === 'PageUp') {
        e.preventDefault();
        goToSlide(currentSlide - 1);
      } else if (e.key === 'Home') {
        e.preventDefault();
        goToSlide(0);
      } else if (e.key === 'End') {
        e.preventDefault();
        goToSlide(totalSlides - 1);
      } else if (e.key === ' ') {
        e.preventDefault();
        setAutoPlay(!autoPlay);
      } else if (e.key === 'o' || e.key === 'O') {
        overviewModal.hidden ? openOverview() : closeOverview();
      } else if (e.key === 'f' || e.key === 'F') {
        fullscreenBtn.click();
      } else if (e.key === 't' || e.key === 'T') {
        themeToggleBtn.click();
      }
    });

    // Touch swipe navigation
    let touchStartX = 0;
    let touchStartY = 0;
    window.addEventListener('touchstart', (e) => {
      touchStartX = e.changedTouches[0].clientX;
      touchStartY = e.changedTouches[0].clientY;
    }, { passive: true });
    window.addEventListener('touchend', (e) => {
      const dx = e.changedTouches[0].clientX - touchStartX;
      const dy = e.changedTouches[0].clientY - touchStartY;
      if (Math.abs(dx) > 55 && Math.abs(dx) > Math.abs(dy) * 1.4) {
        goToSlide(currentSlide + (dx < 0 ? 1 : -1));
      }
    }, { passive: true });
  }

  /* ========================================================================
     Init
     ======================================================================== */
  document.body.classList.add('glow-enabled');
  buildDotsAndOverview();
  buildAccentSwatches();
  applyAccent(ACCENTS[0].hex);
  applyTheme('dark');
  renderRadialWheel();
  bindEvents();

  // Respect URL hash on load (e.g. #slide-2)
  const hashMatch = window.location.hash.match(/^#slide-(\d+)$/);
  if (hashMatch) {
    const idx = parseInt(hashMatch[1], 10) - 1;
    if (idx >= 0 && idx < totalSlides) goToSlide(idx, true);
  }

  rafId = requestAnimationFrame(tickAutoSlide);
})();
