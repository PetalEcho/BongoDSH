// Dependencies are explicit so the build can embed this function without a bundler.
export function mountWidget(model, createArtwork, styles, characters, win = window, desktopBridge = null) {
  const document = win.document;
  const instanceKey = Symbol.for('bongodsh.widget');
  win[instanceKey]?.dispose();
  const storageKey = 'bongodsh:widget:v1';
  const revisionKey = 'bongodsh:desktop-applied-revision:v1';
  const events = new win.AbortController();
  const state = new model.TypingState();
  const animation = new model.Animation();
  let settings = model.settingsFrom(null);
  let storageFailed = false;
  try { settings = model.settingsFrom(JSON.parse(win.localStorage.getItem(storageKey))); }
  catch { storageFailed = true; }
  const consumedRevisions = new Set();
  function revisionsFrom(raw) {
    let value;
    try { value = JSON.parse(raw); } catch { value = raw; }
    return (Array.isArray(value) ? value : [value])
      .filter(item => typeof item === 'string' && item.length > 0 && item.length <= 128).slice(-128);
  }
  function mergeRevisions(revisions) {
    for (const revision of revisions) consumedRevisions.add(revision);
    while (consumedRevisions.size > 128) consumedRevisions.delete(consumedRevisions.values().next().value);
  }
  try { mergeRevisions(revisionsFrom(win.localStorage.getItem(revisionKey))); }
  catch { storageFailed = true; }

  const host = document.createElement('div');
  host.dataset.bongodsh = '';
  host.hidden = true;
  const shadow = host.attachShadow({ mode: 'open' });
  const sheet = document.createElement('style');
  sheet.textContent = styles;
  shadow.append(sheet);
  const pet = document.createElement('button');
  pet.type = 'button';
  pet.className = 'pet';
  pet.setAttribute('aria-label', 'BongoDSH 角色，点击设置或拖动位置');
  pet.setAttribute('aria-expanded', 'false');
  pet.setAttribute('aria-controls', 'bongo-settings');
  pet.title = '拖出窗口移到桌面 · 点击或右键设置';
  const artwork = createArtwork(document, characters);
  pet.append(artwork.svg);
  const hint = document.createElement('span');
  hint.className = 'hint';
  hint.textContent = 'Enter 打开设置';
  const panel = document.createElement('section');
  panel.id = 'bongo-settings';
  panel.className = 'panel';
  panel.hidden = true;
  panel.setAttribute('aria-label', 'BongoDSH 设置');
  panel.innerHTML = `
    <div class="heading" title="拖动移动设置窗口"><strong>BongoDSH</strong><button class="close" type="button" aria-label="关闭设置">×</button></div>
    <label class="row" for="bongo-character">角色</label>
    <select id="bongo-character" name="character"></select>
    <label class="row">暂停动作<input name="paused" type="checkbox"></label>
    <label class="row">跟随输入框<input name="docked" type="checkbox"></label>
    <label class="row" for="bongo-size">角色大小<output for="bongo-size"></output></label>
    <input id="bongo-size" name="size" type="range" min="100" max="220" step="10">
    <label class="row" for="bongo-hand-amplitude">手部幅度<output for="bongo-hand-amplitude"></output></label>
    <input id="bongo-hand-amplitude" name="handAmplitude" type="range" min="0" max="200" step="10">
    <label class="row" for="bongo-body-amplitude">身体幅度<output for="bongo-body-amplitude"></output></label>
    <input id="bongo-body-amplitude" name="bodyAmplitude" type="range" min="0" max="200" step="10">
    <div class="actions"><button type="button" class="action reset">恢复位置</button></div>
    <button type="button" class="action desktop">移到桌面</button>
    <p class="note desktop-status" role="status" hidden></p>
    <p class="note anchor-status" hidden>未找到输入框，暂靠页面边缘。</p>
    <p class="note notice" role="status" hidden>设置未保存，当前页面仍可使用。</p>`;
  shadow.append(pet, hint, panel);
  const characterInput = panel.querySelector('[name=character]');
  for (const character of characters) {
    const option = document.createElement('option');
    option.value = character.id;
    option.textContent = character.label;
    characterInput.append(option);
  }
  const pauseInput = panel.querySelector('[name=paused]');
  const dockInput = panel.querySelector('[name=docked]');
  const sizeInput = panel.querySelector('[name=size]');
  const handAmplitudeInput = panel.querySelector('[name=handAmplitude]');
  const bodyAmplitudeInput = panel.querySelector('[name=bodyAmplitude]');
  const handAmplitudeOutput = panel.querySelector('output[for=bongo-hand-amplitude]');
  const bodyAmplitudeOutput = panel.querySelector('output[for=bongo-body-amplitude]');
  const output = panel.querySelector('output');
  const closeButton = panel.querySelector('.close');
  const panelHeading = panel.querySelector('.heading');
  const notice = panel.querySelector('.notice');
  const anchorStatus = panel.querySelector('.anchor-status');
  const desktopButton = panel.querySelector('.desktop');
  const desktopStatus = panel.querySelector('.desktop-status');
  const motionPreference = win.matchMedia('(prefers-reduced-motion: reduce)');
  let disposed = false;
  let frame = 0;
  let lastFrame = null;
  let layoutFrame = 0;
  let mutationObserver;
  let resizeObserver;
  let composer = null;
  let box = null;
  let drag = null;
  let panelDrag = null;
  let panelPosition = null;
  let suppressClick = false;
  let restoreFocus = null;
  let detached = false;
  let detachPending = false;
  let desktopTimer = 0;
  let desktopVersion = 0;
  let checkingDesktop = false;
  let desktopCheckQueued = false;

  function syncDesktopSettings(status, apply) {
    const next = status?.settings;
    const revision = status?.revision;
    const { handAmplitude = 100, bodyAmplitude = 100 } = next ?? {};
    if (!next || !characters.some(character => character.id === next.character)
      || !Number.isInteger(next.size) || next.size < 100 || next.size > 220 || typeof next.paused !== 'boolean'
      || ![handAmplitude, bodyAmplitude].every(value => Number.isInteger(value) && value >= 0 && value <= 200)
      || typeof revision !== 'string' || !revision.length || revision.length > 128 || consumedRevisions.has(revision)) return;
    let persisted = !storageFailed;
    if (apply) {
      settings.character = next.character;
      settings.size = next.size;
      settings.paused = next.paused;
      settings.handAmplitude = handAmplitude;
      settings.bodyAmplitude = bodyAmplitude;
      desktopVersion++;
      resetInput(); syncSettings();
      persisted = save();
    }
    // Consume stale responses too, so the next poll cannot replay them over a newer page choice.
    mergeRevisions([revision]);
    if (persisted) {
      try { win.localStorage.setItem(revisionKey, JSON.stringify([...consumedRevisions])); }
      catch { storageFailed = true; }
    }
    notice.hidden = !storageFailed;
  }

  function setDetached(value) {
    detached = value;
    if (!value) syncSettings();
    place();
  }

  function setDesktopStatus(message = '') {
    desktopStatus.textContent = message;
    desktopStatus.hidden = !message;
  }

  async function checkDesktop() {
    win.clearTimeout(desktopTimer);
    desktopTimer = 0;
    if (disposed || !desktopBridge || document.hidden) return;
    if (checkingDesktop) { desktopCheckQueued = true; return; }
    checkingDesktop = true;
    desktopCheckQueued = false;
    const version = desktopVersion;
    try {
      const status = await desktopBridge.status();
      if (disposed) return;
      const current = !detachPending && version === desktopVersion;
      syncDesktopSettings(status, current);
      if (current) setDetached(status.active === true);
      else if (!detachPending) desktopCheckQueued = true;
      setDesktopStatus(status.available ? '' : '未安装桌面组件，请按插件说明设置。');
    } catch {
      if (!disposed) {
        setDesktopStatus('桌面连接不可用，请检查本机 DSH 和桌面组件。');
      }
    } finally { checkingDesktop = false; }
    if (!disposed && !document.hidden && !detachPending && (desktopCheckQueued || detached)) {
      win.clearTimeout(desktopTimer);
      desktopTimer = win.setTimeout(checkDesktop, desktopCheckQueued ? 0 : 1000);
    }
  }

  async function detachToDesktop(grabX = 0.5, grabY = 0.5) {
    if (detachPending || detached || disposed) return;
    if (!desktopBridge) {
      setDesktopStatus('请在本机 DSH 中移到桌面。');
      openPanel();
      return;
    }
    detachPending = true;
    const version = ++desktopVersion;
    let stale = false;
    desktopButton.disabled = true;
    setDesktopStatus('正在移到桌面…');
    resetInput();
    try {
      const status = await desktopBridge.detach(settings.character, settings.size, grabX, grabY,
        settings.paused, settings.handAmplitude, settings.bodyAmplitude);
      if (disposed) return;
      stale = version !== desktopVersion;
      syncDesktopSettings(status, !stale);
      endDrag(true);
      if (!stale) setDetached(status.active === true);
    } catch (error) {
      if (disposed) return;
      endDrag(true);
      stale = version !== desktopVersion;
      if (!stale) {
        setDesktopStatus(error.message || '移到桌面失败，请重试。');
        if (!document.hidden) openPanel(false);
      }
    } finally {
      detachPending = false;
      desktopButton.disabled = false;
      if (!disposed && !document.hidden) {
        if (stale || desktopCheckQueued) void checkDesktop();
        else if (detached) {
          win.clearTimeout(desktopTimer);
          desktopTimer = win.setTimeout(checkDesktop, 1000);
        }
      }
    }
  }

  function on(target, name, listener, options = {}) {
    target.addEventListener(name, listener, { ...options, signal: events.signal });
  }

  function save() {
    try {
      win.localStorage.setItem(storageKey, JSON.stringify(settings));
      storageFailed = false;
    } catch { storageFailed = true; }
    notice.hidden = !storageFailed;
    return !storageFailed;
  }

  function syncSettings() {
    if (artwork.svg.dataset.character !== settings.character) {
      const character = artwork.select(settings.character);
      pet.setAttribute('aria-label', `BongoDSH ${character.label}，点击设置或拖动位置`);
    }
    characterInput.value = settings.character;
    pauseInput.checked = settings.paused;
    dockInput.checked = settings.docked;
    sizeInput.value = String(settings.size);
    sizeInput.style.setProperty('--range-progress', `${(settings.size - 100) / 120 * 100}%`);
    output.textContent = `${settings.size} px`;
    handAmplitudeInput.value = String(settings.handAmplitude);
    bodyAmplitudeInput.value = String(settings.bodyAmplitude);
    handAmplitudeInput.style.setProperty('--range-progress', `${settings.handAmplitude / 2}%`);
    bodyAmplitudeInput.style.setProperty('--range-progress', `${settings.bodyAmplitude / 2}%`);
    handAmplitudeOutput.textContent = `${settings.handAmplitude}%`;
    bodyAmplitudeOutput.textContent = `${settings.bodyAmplitude}%`;
    pet.dataset.paused = String(settings.paused);
    notice.hidden = !storageFailed;
  }

  function viewport() {
    const visual = win.visualViewport;
    return {
      left: visual?.offsetLeft ?? 0, top: visual?.offsetTop ?? 0,
      width: visual?.width ?? win.innerWidth, height: visual?.height ?? win.innerHeight,
    };
  }

  function visibleRect(element) {
    if (!element?.isConnected || element.closest('[hidden], [inert], [aria-hidden="true"]')) return null;
    const rect = element.getBoundingClientRect();
    const view = viewport();
    if (rect.width < 40 || rect.height < 16 || rect.bottom <= view.top || rect.top >= view.top + view.height
      || rect.right <= view.left || rect.left >= view.left + view.width) return null;
    const style = win.getComputedStyle(element);
    return style.visibility === 'hidden' || style.display === 'none' ? null : rect;
  }

  function locateComposer() {
    // This is DSH's explicit composer marker; do not infer an input from its text.
    const candidates = document.querySelectorAll('[data-composer-card]');
    const focused = document.activeElement?.closest?.('[data-composer-card]');
    const eligible = element => visibleRect(element) && conversationActive(element);
    const next = focused && eligible(focused) ? focused
      : [...candidates].filter(eligible).at(-1) ?? null;
    if (next !== composer) {
      if (composer) resizeObserver?.unobserve(composer);
      composer = next;
      if (composer) resizeObserver?.observe(composer);
    }
    return visibleRect(composer);
  }

  function conversationActive(element) {
    // DSH keeps the composer in other views and pins Chat as the first tab.
    // Use the nearest conversation tab row, without depending on translated labels.
    for (let parent = element.parentElement; parent; parent = parent.parentElement) {
      const tabs = parent.querySelector('[data-conversation-tabs]');
      if (tabs) return tabs.querySelector('[role="tab"]')?.getAttribute('aria-selected') === 'true';
    }
    // New conversations and the standalone preview have no tab row.
    return true;
  }

  function positionPanel() {
    if (panel.hidden || !box) return;
    const view = viewport();
    panel.style.maxHeight = `${Math.max(60, view.height - 16)}px`;
    const width = panel.offsetWidth;
    const height = panel.offsetHeight;
    const x = model.clamp(panelPosition?.x ?? box.x + box.width - width, view.left + 8,
      Math.max(view.left + 8, view.left + view.width - width - 8));
    const y = model.clamp(panelPosition?.y ?? box.y - height - 8, view.top + 8,
      Math.max(view.top + 8, view.top + view.height - height - 8));
    if (panelPosition) panelPosition = { x, y };
    panel.style.left = `${x - box.x}px`;
    panel.style.top = `${y - box.y}px`;
  }

  function place() {
    if (disposed) return;
    const rect = locateComposer();
    // Only a visible composer belonging to the active Chat view can host the pet.
    const hidden = detached || !rect;
    if (host.hidden !== hidden) {
      host.hidden = hidden;
      if (hidden) { resetInput(); closePanel(); endDrag(true); }
    }
    if (hidden || drag) return;
    box = model.placement(settings, viewport(), rect);
    host.style.width = `${box.width}px`;
    host.style.height = `${box.height}px`;
    host.style.transform = `translate(${box.x}px, ${box.y}px)`;
    anchorStatus.hidden = !settings.docked || !!rect;
    positionPanel();
  }

  function schedulePlace() {
    if (disposed || document.hidden || layoutFrame) return;
    layoutFrame = win.requestAnimationFrame(() => { layoutFrame = 0; place(); });
  }

  function render() {
    artwork.render(animation.positions(motionPreference.matches), settings.handAmplitude, settings.bodyAmplitude);
  }

  function animate(now) {
    frame = 0;
    if (disposed || document.hidden || host.hidden) return;
    const moving = animation.advance(lastFrame === null ? 1 / 60 : (now - lastFrame) / 1000);
    lastFrame = now;
    render();
    if (moving && !motionPreference.matches) frame = win.requestAnimationFrame(animate);
    else lastFrame = null;
  }

  function apply(feedback) {
    animation.apply(feedback);
    if (motionPreference.matches) render();
    else if (!frame && !document.hidden && !host.hidden && !disposed) frame = win.requestAnimationFrame(animate);
  }

  function resetInput() {
    if (frame) win.cancelAnimationFrame(frame);
    frame = 0;
    lastFrame = null;
    state.reset();
    animation.reset();
    render();
  }

  function closePanel(restore = false) {
    endPanelDrag();
    if (panel.hidden) return;
    panel.hidden = true;
    pet.setAttribute('aria-expanded', 'false');
    if (restore && restoreFocus?.isConnected) restoreFocus.focus({ preventScroll: true });
    restoreFocus = null;
  }

  function openPanel(refreshDesktop = true) {
    if (disposed || host.hidden) return;
    resetInput();
    restoreFocus = document.activeElement === host ? pet : document.activeElement;
    panel.hidden = false;
    pet.setAttribute('aria-expanded', 'true');
    positionPanel();
    closeButton.focus({ preventScroll: true });
    if (refreshDesktop) void checkDesktop();
  }

  function ownEvent(event) { return event.composedPath().includes(host); }

  function passwordEvent(event) {
    return event.composedPath().some(element => element.matches?.('input[type="password"]'));
  }

  function endDrag(cancelled = false) {
    if (!drag) return;
    const previous = drag;
    drag = null;
    if (pet.hasPointerCapture(previous.id)) pet.releasePointerCapture(previous.id);
    suppressClick = previous.moved;
    if (previous.moved && !cancelled) {
      const rect = locateComposer();
      const dockBox = model.placement({ ...settings, docked: true }, viewport(), rect);
      settings.docked = !!rect && Math.hypot(box.x - dockBox.x, box.y - dockBox.y) < 36;
      settings.x = box.maxX === box.minX ? 0 : (box.x - box.minX) / (box.maxX - box.minX);
      settings.y = box.maxY === box.minY ? 0 : (box.y - box.minY) / (box.maxY - box.minY);
      save();
      syncSettings();
    }
    place();
  }

  function endPanelDrag() {
    if (!panelDrag) return;
    const { id } = panelDrag;
    panelDrag = null;
    if (panelHeading.hasPointerCapture(id)) panelHeading.releasePointerCapture(id);
  }

  on(panelHeading, 'pointerdown', event => {
    if (event.button !== 0 || !event.isPrimary || event.target.closest('button')) return;
    const rect = panel.getBoundingClientRect();
    panelDrag = { id: event.pointerId, startX: event.clientX, startY: event.clientY, x: rect.left, y: rect.top };
    panelHeading.setPointerCapture(event.pointerId);
    event.preventDefault();
  });
  on(panelHeading, 'pointermove', event => {
    if (!panelDrag || event.pointerId !== panelDrag.id) return;
    panelPosition = {
      x: panelDrag.x + event.clientX - panelDrag.startX,
      y: panelDrag.y + event.clientY - panelDrag.startY,
    };
    positionPanel();
  });
  on(panelHeading, 'pointerup', event => { if (event.pointerId === panelDrag?.id) endPanelDrag(); });
  on(panelHeading, 'pointercancel', endPanelDrag);
  on(panelHeading, 'lostpointercapture', endPanelDrag);

  on(pet, 'pointerdown', event => {
    if (event.button !== 0 || !event.isPrimary || !box) return;
    suppressClick = false;
    drag = {
      id: event.pointerId, startX: event.clientX, startY: event.clientY, x: box.x, y: box.y, moved: false,
      grabX: model.clamp((event.clientX - box.x) / box.width, 0, 1),
      grabY: model.clamp((event.clientY - box.y) / box.height, 0, 1), leaving: false,
    };
    pet.setPointerCapture(event.pointerId);
    // Dragging the companion must not take keyboard focus from the composer.
    event.preventDefault();
  });
  on(pet, 'pointermove', event => {
    if (!drag || event.pointerId !== drag.id) return;
    const dx = event.clientX - drag.startX;
    const dy = event.clientY - drag.startY;
    if (!drag.moved && Math.hypot(dx, dy) < 4) return;
    drag.moved = true;
    closePanel();
    box.x = model.clamp(drag.x + dx, box.minX, box.maxX);
    box.y = model.clamp(drag.y + dy, box.minY, box.maxY);
    host.style.transform = `translate(${box.x}px, ${box.y}px)`;
    const view = viewport();
    // Electron may stop delivering moves after the pointer leaves the native window.
    const outside = model.leavesViewport({ ...box, x: drag.x + dx, y: drag.y + dy }, view,
      { x: event.clientX, y: event.clientY }, { x: dx, y: dy });
    if (!drag.leaving && outside) {
      drag.leaving = true;
      void detachToDesktop(drag.grabX, drag.grabY);
    }
  });
  on(pet, 'pointerup', event => { if (event.pointerId === drag?.id) endDrag(); });
  // The native window can cancel page pointer capture while taking over the drag.
  on(pet, 'pointercancel', () => endDrag(true));
  on(pet, 'lostpointercapture', () => endDrag(true));
  on(pet, 'click', event => {
    if (suppressClick && event.detail !== 0) { suppressClick = false; return; }
    if (panel.hidden) openPanel(); else closePanel(true);
  });
  on(pet, 'contextmenu', event => { event.preventDefault(); endDrag(true); openPanel(); });
  on(closeButton, 'click', () => closePanel(true));
  on(document, 'pointerdown', event => { if (!ownEvent(event)) closePanel(); }, { capture: true });
  on(panel, 'keydown', event => {
    if (event.code === 'Escape') { event.stopPropagation(); closePanel(true); }
  });
  on(characterInput, 'change', () => {
    settings.character = characterInput.value;
    desktopVersion++;
    resetInput(); save(); syncSettings(); place();
  });
  on(pauseInput, 'change', () => {
    settings.paused = pauseInput.checked;
    desktopVersion++;
    resetInput(); save(); syncSettings();
  });
  on(dockInput, 'change', () => { settings.docked = dockInput.checked; save(); place(); });
  on(sizeInput, 'input', () => {
    settings.size = model.clamp(Number(sizeInput.value), 100, 220);
    desktopVersion++;
    syncSettings(); place();
  });
  on(sizeInput, 'change', save);
  for (const [input, name] of [[handAmplitudeInput, 'handAmplitude'], [bodyAmplitudeInput, 'bodyAmplitude']]) {
    on(input, 'input', () => {
      const value = Number(input.value);
      if (!Number.isInteger(value)) return;
      settings[name] = model.clamp(value, 0, 200);
      desktopVersion++;
      syncSettings(); render(); save();
    });
  }
  on(panel.querySelector('.reset'), 'click', () => {
    settings.docked = true; settings.x = 1; settings.y = 1;
    save(); syncSettings(); place();
  });
  on(desktopButton, 'click', () => { void detachToDesktop(); });
  on(win, 'keydown', event => {
    if (!event.isTrusted || document.hidden || host.hidden || detachPending || settings.paused || !panel.hidden || ownEvent(event) || passwordEvent(event)) return;
    if (event.metaKey || ((event.ctrlKey || event.altKey) && !event.getModifierState('AltGraph'))) {
      resetInput();
      return;
    }
    if (model.typingCodes.test(event.code)) apply(state.press(event.code, event.repeat));
  }, { capture: true });
  // Always accept releases, including while a shortcut modifier is held.
  on(win, 'keyup', event => { if (state.held.has(event.code)) apply(state.release(event.code)); }, { capture: true });
  on(win, 'blur', () => { resetInput(); endDrag(true); closePanel(); });
  on(win, 'pagehide', () => { resetInput(); endDrag(true); endPanelDrag(); });
  on(win, 'pageshow', schedulePlace);
  on(win, 'focus', () => { void checkDesktop(); });
  on(document, 'visibilitychange', () => {
    resetInput(); endDrag(true);
    if (document.hidden) {
      if (layoutFrame) win.cancelAnimationFrame(layoutFrame);
      layoutFrame = 0;
      closePanel();
      win.clearTimeout(desktopTimer);
      desktopTimer = 0;
    } else { schedulePlace(); void checkDesktop(); }
  });
  on(motionPreference, 'change', resetInput);
  on(win, 'resize', () => { endDrag(true); endPanelDrag(); schedulePlace(); });
  on(document, 'scroll', schedulePlace, { capture: true, passive: true });
  on(document, 'focusin', schedulePlace);
  on(document, 'transitionend', schedulePlace, { capture: true });
  on(win, 'popstate', () => { resetInput(); schedulePlace(); });
  on(win, 'hashchange', () => { resetInput(); schedulePlace(); });
  if (win.visualViewport) {
    on(win.visualViewport, 'resize', schedulePlace);
    on(win.visualViewport, 'scroll', schedulePlace);
  }
  on(win, 'storage', event => {
    if (event.key === revisionKey) {
      mergeRevisions(revisionsFrom(event.newValue));
      return;
    }
    if (event.key !== storageKey && event.key !== null) return;
    try { settings = model.settingsFrom(JSON.parse(event.newValue)); }
    catch { return; }
    desktopVersion++;
    resetInput(); endDrag(true); syncSettings(); schedulePlace();
  });

  function start() {
    if (disposed) return;
    document.body.append(host);
    resizeObserver = new win.ResizeObserver(schedulePlace);
    resizeObserver.observe(document.documentElement);
    resizeObserver.observe(panel);
    mutationObserver = new win.MutationObserver(records => {
      // Ignore our own positioning writes and avoid an observer/repaint loop.
      if (records.some(record => record.target !== host && !host.contains(record.target))) schedulePlace();
    });
    mutationObserver.observe(document.body, {
      subtree: true, childList: true, attributes: true,
      attributeFilter: ['hidden', 'inert', 'aria-hidden', 'aria-selected', 'class', 'style', 'data-composer-card'],
    });
    syncSettings(); place(); render();
    void checkDesktop();
  }

  function dispose() {
    if (disposed) return;
    disposed = true;
    events.abort();
    resetInput();
    endDrag(true);
    endPanelDrag();
    if (layoutFrame) win.cancelAnimationFrame(layoutFrame);
    resizeObserver?.disconnect();
    mutationObserver?.disconnect();
    win.clearTimeout(desktopTimer);
    desktopBridge?.dispose();
    if (document.activeElement === host && restoreFocus?.isConnected) restoreFocus.focus({ preventScroll: true });
    host.remove();
    if (win[instanceKey] === controller) delete win[instanceKey];
  }

  const controller = { dispose };
  win[instanceKey] = controller;
  if (document.body) start(); else on(document, 'DOMContentLoaded', start, { once: true });
  return controller;
}
