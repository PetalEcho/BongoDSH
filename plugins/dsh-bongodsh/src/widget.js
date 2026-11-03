// Dependencies are explicit so the build can embed this function without a bundler.
export function mountWidget(model, createArtwork, styles, characters, win = window) {
  const document = win.document;
  const instanceKey = Symbol.for('bongodsh.widget');
  win[instanceKey]?.dispose();
  const storageKey = 'bongodsh:widget:v1';
  const events = new win.AbortController();
  const state = new model.TypingState();
  const animation = new model.Animation();
  let settings = model.settingsFrom(null);
  let storageFailed = false;
  try { settings = model.settingsFrom(JSON.parse(win.localStorage.getItem(storageKey))); }
  catch { storageFailed = true; }

  const host = document.createElement('div');
  host.dataset.bongodsh = '';
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
  pet.title = '拖动移动 · 点击或右键设置';
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
    <div class="heading"><strong>BongoDSH</strong><button class="close" type="button" aria-label="关闭设置">×</button></div>
    <p>打字时，角色随你的节奏轻轻动一动。</p>
    <label class="row" for="bongo-character">角色</label>
    <select id="bongo-character" name="character"></select>
    <p class="note character-note"></p>
    <label class="row">暂停动作<input name="paused" type="checkbox"></label>
    <label class="row">跟随输入框<input name="docked" type="checkbox"></label>
    <label class="row" for="bongo-size">角色大小<output for="bongo-size"></output></label>
    <input id="bongo-size" name="size" type="range" min="100" max="220" step="10">
    <div class="actions"><button type="button" class="action demo">试一下</button><button type="button" class="action reset">恢复位置</button></div>
    <p class="note anchor-status"></p>
    <p class="note">仅响应当前页面的物理按键，不保存输入内容。</p>
    <p class="note notice" role="status" hidden>设置暂时无法保存，当前页面仍可使用。</p>`;
  shadow.append(pet, hint, panel);
  const characterInput = panel.querySelector('[name=character]');
  const characterNote = panel.querySelector('.character-note');
  for (const character of characters) {
    const option = document.createElement('option');
    option.value = character.id;
    option.textContent = character.label;
    characterInput.append(option);
  }
  const pauseInput = panel.querySelector('[name=paused]');
  const dockInput = panel.querySelector('[name=docked]');
  const sizeInput = panel.querySelector('[name=size]');
  const output = panel.querySelector('output');
  const closeButton = panel.querySelector('.close');
  const notice = panel.querySelector('.notice');
  const anchorStatus = panel.querySelector('.anchor-status');
  const motionPreference = win.matchMedia('(prefers-reduced-motion: reduce)');
  let disposed = false;
  let frame = 0;
  let lastFrame = null;
  let demoTimer = 0;
  let layoutFrame = 0;
  let mutationObserver;
  let resizeObserver;
  let composer = null;
  let box = null;
  let drag = null;
  let suppressClick = false;
  let restoreFocus = null;

  function on(target, name, listener, options = {}) {
    target.addEventListener(name, listener, { ...options, signal: events.signal });
  }

  function save() {
    try {
      win.localStorage.setItem(storageKey, JSON.stringify(settings));
      storageFailed = false;
    } catch { storageFailed = true; }
    notice.hidden = !storageFailed;
  }

  function syncSettings() {
    if (artwork.svg.dataset.character !== settings.character) {
      const character = artwork.select(settings.character);
      pet.setAttribute('aria-label', `BongoDSH ${character.label}，点击设置或拖动位置`);
      characterNote.textContent = character.leftHand || character.rightHand
        ? '身体与双手独立动作。' : '随按键轻轻压下身体。';
    }
    characterInput.value = settings.character;
    pauseInput.checked = settings.paused;
    dockInput.checked = settings.docked;
    sizeInput.value = String(settings.size);
    output.textContent = `${settings.size} px`;
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
    const next = focused && visibleRect(focused) ? focused
      : [...candidates].filter(element => visibleRect(element)).at(-1) ?? null;
    if (next !== composer) {
      if (composer) resizeObserver?.unobserve(composer);
      composer = next;
      if (composer) resizeObserver?.observe(composer);
    }
    return visibleRect(composer);
  }

  function positionPanel() {
    if (panel.hidden || !box) return;
    const view = viewport();
    panel.style.maxHeight = `${Math.max(60, view.height - 16)}px`;
    const width = panel.offsetWidth;
    const height = panel.offsetHeight;
    const x = model.clamp(box.x + box.width - width, view.left + 8,
      Math.max(view.left + 8, view.left + view.width - width - 8));
    const y = model.clamp(box.y - height - 8, view.top + 8,
      Math.max(view.top + 8, view.top + view.height - height - 8));
    panel.style.left = `${x - box.x}px`;
    panel.style.top = `${y - box.y}px`;
  }

  function place() {
    if (disposed || drag) return;
    const rect = locateComposer();
    box = model.placement(settings, viewport(), rect);
    host.style.width = `${box.width}px`;
    host.style.height = `${box.height}px`;
    host.style.transform = `translate(${box.x}px, ${box.y}px)`;
    anchorStatus.textContent = settings.docked
      ? (rect ? '已跟随输入框上沿。' : '未找到可见输入框，暂时停靠页面边缘。')
      : '自由放置，可拖动角色调整位置。';
    positionPanel();
  }

  function schedulePlace() {
    if (disposed || document.hidden || layoutFrame) return;
    layoutFrame = win.requestAnimationFrame(() => { layoutFrame = 0; place(); });
  }

  function render() {
    artwork.render(animation.positions(motionPreference.matches));
  }

  function animate(now) {
    frame = 0;
    if (disposed || document.hidden) return;
    const moving = animation.advance(lastFrame === null ? 1 / 60 : (now - lastFrame) / 1000);
    lastFrame = now;
    render();
    if (moving && !motionPreference.matches) frame = win.requestAnimationFrame(animate);
    else lastFrame = null;
  }

  function apply(feedback) {
    animation.apply(feedback);
    if (motionPreference.matches) render();
    else if (!frame && !document.hidden && !disposed) frame = win.requestAnimationFrame(animate);
  }

  function resetInput() {
    win.clearTimeout(demoTimer);
    demoTimer = 0;
    if (frame) win.cancelAnimationFrame(frame);
    frame = 0;
    lastFrame = null;
    state.reset();
    animation.reset();
    render();
  }

  function closePanel(restore = false) {
    if (panel.hidden) return;
    panel.hidden = true;
    pet.setAttribute('aria-expanded', 'false');
    if (restore && restoreFocus?.isConnected) restoreFocus.focus({ preventScroll: true });
    restoreFocus = null;
  }

  function openPanel() {
    resetInput();
    restoreFocus = document.activeElement === host ? pet : document.activeElement;
    panel.hidden = false;
    pet.setAttribute('aria-expanded', 'true');
    positionPanel();
    closeButton.focus({ preventScroll: true });
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

  on(pet, 'pointerdown', event => {
    if (event.button !== 0 || !event.isPrimary || !box) return;
    suppressClick = false;
    drag = { id: event.pointerId, startX: event.clientX, startY: event.clientY, x: box.x, y: box.y, moved: false };
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
  });
  on(pet, 'pointerup', event => { if (event.pointerId === drag?.id) endDrag(); });
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
    resetInput(); save(); syncSettings(); place();
  });
  on(pauseInput, 'change', () => {
    settings.paused = pauseInput.checked;
    resetInput(); save(); syncSettings();
  });
  on(dockInput, 'change', () => { settings.docked = dockInput.checked; save(); place(); });
  on(sizeInput, 'input', () => {
    settings.size = model.clamp(Number(sizeInput.value), 100, 220);
    syncSettings(); place();
  });
  on(sizeInput, 'change', save);
  on(panel.querySelector('.reset'), 'click', () => {
    settings.docked = true; settings.x = 1; settings.y = 1;
    save(); syncSettings(); place();
  });
  on(panel.querySelector('.demo'), 'click', () => {
    resetInput();
    apply(state.press('KeyA')); apply(state.release('KeyA'));
    demoTimer = win.setTimeout(() => {
      apply(state.press('KeyB')); apply(state.release('KeyB'));
      demoTimer = 0;
    }, 120);
  });
  on(win, 'keydown', event => {
    if (!event.isTrusted || document.hidden || settings.paused || !panel.hidden || ownEvent(event) || passwordEvent(event)) return;
    if (event.metaKey || ((event.ctrlKey || event.altKey) && !event.getModifierState('AltGraph'))) {
      resetInput();
      return;
    }
    if (model.typingCodes.test(event.code)) apply(state.press(event.code, event.repeat));
  }, { capture: true });
  // Always accept releases, including while a shortcut modifier is held.
  on(win, 'keyup', event => { if (state.held.has(event.code)) apply(state.release(event.code)); }, { capture: true });
  on(win, 'blur', () => { resetInput(); endDrag(true); closePanel(); });
  on(win, 'pagehide', () => { resetInput(); endDrag(true); });
  on(win, 'pageshow', schedulePlace);
  on(document, 'visibilitychange', () => {
    resetInput(); endDrag(true);
    if (document.hidden) {
      if (layoutFrame) win.cancelAnimationFrame(layoutFrame);
      layoutFrame = 0;
      closePanel();
    } else schedulePlace();
  });
  on(motionPreference, 'change', resetInput);
  on(win, 'resize', () => { endDrag(true); schedulePlace(); });
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
    if (event.key !== storageKey && event.key !== null) return;
    try { settings = model.settingsFrom(JSON.parse(event.newValue)); }
    catch { return; }
    resetInput(); endDrag(true); syncSettings(); schedulePlace();
  });

  function start() {
    if (disposed) return;
    document.body.append(host);
    resizeObserver = new win.ResizeObserver(schedulePlace);
    resizeObserver.observe(document.documentElement);
    mutationObserver = new win.MutationObserver(records => {
      // Ignore our own positioning writes and avoid an observer/repaint loop.
      if (records.some(record => record.target !== host && !host.contains(record.target))) schedulePlace();
    });
    mutationObserver.observe(document.body, {
      subtree: true, childList: true, attributes: true,
      attributeFilter: ['hidden', 'inert', 'aria-hidden', 'class', 'style', 'data-composer-card'],
    });
    syncSettings(); place(); render();
  }

  function dispose() {
    if (disposed) return;
    disposed = true;
    events.abort();
    resetInput();
    endDrag(true);
    if (layoutFrame) win.cancelAnimationFrame(layoutFrame);
    resizeObserver?.disconnect();
    mutationObserver?.disconnect();
    if (document.activeElement === host && restoreFocus?.isConnected) restoreFocus.focus({ preventScroll: true });
    host.remove();
    if (win[instanceKey] === controller) delete win[instanceKey];
  }

  const controller = { dispose };
  win[instanceKey] = controller;
  if (document.body) start(); else on(document, 'DOMContentLoaded', start, { once: true });
  return controller;
}
