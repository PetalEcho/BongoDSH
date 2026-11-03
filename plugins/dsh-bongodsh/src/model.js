// Keep this factory self-contained: the dependency-free build embeds it verbatim.
export function createModel(characterIds = ['fish-maid']) {
  const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
  const typingCodes = /^(Key[A-Z]|Digit[0-9]|Numpad[0-9]|Numpad(?:Add|Subtract|Multiply|Divide|Decimal|Enter|Equal)|Space|Enter|Backspace|Tab|Minus|Equal|BracketLeft|BracketRight|Backslash|Semicolon|Quote|Backquote|Comma|Period|Slash|IntlBackslash|IntlRo|IntlYen)$/;

  class TypingState {
    constructor() {
      this.held = new Map();
      this.next = 0;
    }

    press(code, repeat = false) {
      let strike = null;
      if (!repeat && typingCodes.test(code) && !this.held.has(code)) {
        strike = this.next;
        this.held.set(code, strike);
        this.next = 1 - this.next;
      }
      return this.feedback(strike);
    }

    release(code) {
      this.held.delete(code);
      return this.feedback();
    }

    reset() {
      this.held.clear();
      this.next = 0;
      return this.feedback();
    }

    feedback(strike = null) {
      const pose = [false, false];
      for (const hand of this.held.values()) pose[hand] = true;
      return { pose, strike };
    }
  }

  class Animation {
    constructor() {
      this.reset();
    }

    reset() {
      this.pose = [false, false];
      this.springs = [0, 1].map(() => ({ position: 0, velocity: 0, tap: 0 }));
    }

    apply({ pose, strike }) {
      this.pose = pose;
      if (strike !== null) {
        const spring = this.springs[strike];
        spring.tap = 0.075;
        spring.velocity = Math.max(8, spring.velocity);
      }
    }

    advance(seconds) {
      let remaining = Number.isFinite(seconds) ? clamp(seconds, 0, 0.05) : 0;
      while (remaining > 0) {
        const dt = Math.min(remaining, 1 / 120);
        this.springs.forEach((spring, hand) => {
          const target = this.pose[hand] || spring.tap > 0 ? 1 : 0;
          spring.tap = Math.max(0, spring.tap - dt);
          spring.velocity += (520 * (target - spring.position) - 27 * spring.velocity) * dt;
          spring.position += spring.velocity * dt;
          if (Math.abs(spring.position - target) < 0.001 && Math.abs(spring.velocity) < 0.01) {
            spring.position = target;
            spring.velocity = 0;
          }
        });
        remaining = Math.max(0, remaining - dt);
      }
      return this.springs.some((spring, hand) => spring.tap > 0
        || spring.position !== Number(this.pose[hand]) || spring.velocity !== 0);
    }

    positions(reduced = false) {
      return reduced ? this.pose.map(Number) : this.springs.map(spring => spring.position);
    }
  }

  function settingsFrom(value) {
    const source = value && value.version === 1 ? value : {};
    return {
      version: 1,
      character: characterIds.includes(source.character) ? source.character : characterIds[0],
      size: Number.isFinite(source.size) ? clamp(source.size, 100, 220) : 160,
      paused: source.paused === true,
      docked: source.docked !== false,
      x: Number.isFinite(source.x) ? clamp(source.x, 0, 1) : 1,
      y: Number.isFinite(source.y) ? clamp(source.y, 0, 1) : 1,
    };
  }

  function placement(settings, viewport, composer = null) {
    const margin = Math.min(8, viewport.width / 4, viewport.height / 4);
    const width = Math.min(settings.size, Math.max(1, viewport.width - margin * 2),
      Math.max(1, (viewport.height - margin * 2) / 0.84));
    const height = width * 0.84;
    const minX = viewport.left + margin;
    const minY = viewport.top + margin;
    const maxX = Math.max(minX, viewport.left + viewport.width - width - margin);
    const maxY = Math.max(minY, viewport.top + viewport.height - height - margin);
    let x = minX + settings.x * (maxX - minX);
    let y = minY + settings.y * (maxY - minY);
    if (settings.docked && composer) {
      x = composer.right - width - 8;
      // The visible body ends at y=248 in the 300×252 artwork, not at the SVG box edge.
      y = composer.top - width * 248 / 300 - 1;
    }
    return { x: clamp(x, minX, maxX), y: clamp(y, minY, maxY), width, height, minX, minY, maxX, maxY };
  }

  function leavesViewport(box, viewport, pointer = null, movement = null) {
    const overshoot = 16;
    return box.x < viewport.left - overshoot || box.y < viewport.top - overshoot
      || box.x + box.width > viewport.left + viewport.width + overshoot
      || box.y + box.height > viewport.top + viewport.height + overshoot
      || !!(pointer && movement && (
        (movement.x < 0 && pointer.x <= viewport.left + 8)
        || (movement.y < 0 && pointer.y <= viewport.top + 8)
        || (movement.x > 0 && pointer.x >= viewport.left + viewport.width - 8)
        || (movement.y > 0 && pointer.y >= viewport.top + viewport.height - 8)));
  }

  return { TypingState, Animation, settingsFrom, placement, leavesViewport, clamp, typingCodes };
}
