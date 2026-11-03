export const styles = `
  :host { all: initial; position: fixed; left: 0; top: 0; z-index: 40; pointer-events: none;
    font: 13px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", sans-serif; color: #2a4352; }
  *, *::before, *::after { box-sizing: border-box; }
  [hidden] { display: none !important; }
  button, input, select { font: inherit; }
  button { cursor: pointer; color: inherit; }
  button:focus-visible, input:focus-visible, select:focus-visible { outline: 2px solid #27768b; outline-offset: 3px; }
  .pet { display: block; width: 100%; height: 100%; padding: 0; border: 0; background: transparent;
    pointer-events: auto; touch-action: none; user-select: none; -webkit-user-select: none; cursor: grab; border-radius: 45%; }
  .pet:active { cursor: grabbing; }
  .pet svg { width: 100%; height: 100%; display: block; overflow: visible; pointer-events: none; }
  .pet[data-paused="true"] { opacity: .62; }
  .hint { position: absolute; right: 0; bottom: -22px; white-space: nowrap; background: #ebf8f6;
    border: 1px solid #c7dfe3; border-radius: 6px; padding: 1px 6px; font-size: 11px;
    opacity: 0; transition: opacity .12s; }
  .pet:focus-visible + .hint { opacity: 1; }
  .panel { position: absolute; width: 248px; max-width: calc(100vw - 16px); padding: 14px;
    background: #f5fbfc; color: #2a4352; border: 1px solid #b7d4db; border-radius: 12px;
    box-shadow: 0 5px 24px #1b455a26; pointer-events: auto; overflow-y: auto; }
  .heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 4px; }
  .heading strong { font-size: 15px; font-weight: 650; }
  .close { border: 0; background: transparent; padding: 2px 6px; font-size: 19px; line-height: 1; }
  p { margin: 0 0 10px; font-size: 12px; color: #536b77; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 10px; min-height: 36px; }
  .row input[type=checkbox] { width: 16px; height: 16px; accent-color: #27768b; }
  select { width: 100%; padding: 6px 8px; border: 1px solid #bad5dc; border-radius: 7px;
    background: #fff; color: inherit; }
  .character-note { margin: 6px 0; }
  input[type=range] { display: block; width: 100%; margin: 6px 0 14px; accent-color: #27768b; }
  output { font-variant-numeric: tabular-nums; color: #536b77; }
  .actions { display: flex; gap: 8px; margin: 10px 0; }
  .action { flex: 1; background: #e3f1f4; border: 1px solid #bad5dc; border-radius: 7px; padding: 6px 8px; }
  .note { margin: 10px 0 0; font-size: 11px; color: #536b77; }
  .notice { color: #9b492b; }
  @media (prefers-color-scheme: dark) {
    .panel { background: #243b46; color: #e4f3f6; border-color: #537581; }
    p, output, .note { color: #b6ccd5; }
    .action { background: #345360; border-color: #5a7b88; color: #e4f3f6; }
    select { background: #345360; border-color: #5a7b88; }
    .notice { color: #f5bea0; }
  }
  @media (prefers-reduced-motion: reduce) { .hint { transition: none; } }
`;
