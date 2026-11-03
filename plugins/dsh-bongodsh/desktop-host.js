import { spawn } from 'node:child_process';
import { access } from 'node:fs/promises';
import { constants } from 'node:fs';
import { fileURLToPath } from 'node:url';

const characterIds = new Set(['fish-maid', 'sailor', 'pajamas', 'wizard', 'blue-maid', 'orange-bride']);
const endpoint = 'bongodsh/desktop';

async function findExecutable() {
  if (!['darwin', 'win32'].includes(process.platform)) return null;
  const name = process.platform === 'win32' ? 'bongodsh.exe' : 'bongodsh';
  const candidates = [
    ...(process.platform === 'darwin'
      ? [new URL(`./native/darwin-${process.arch}/BongoDSH Companion.app/Contents/MacOS/bongodsh`, import.meta.url)] : []),
    new URL(`./native/${process.platform}-${process.arch}/${name}`, import.meta.url),
    new URL(`../../target/release/${name}`, import.meta.url),
  ];
  for (const candidate of candidates) {
    try {
      await access(candidate, process.platform === 'win32' ? constants.F_OK : constants.X_OK);
      return fileURLToPath(candidate);
    } catch { /* A source checkout may not have been built yet. */ }
  }
  return null;
}

export function createDesktopHost({ resolveExecutable = findExecutable, spawnProcess = spawn } = {}) {
  let session = null;
  let disposed = false;

  async function stop() {
    const current = session;
    if (!current) return;
    current.closing = true;
    current.child.stdin.end('{"type":"close"}\n');
    const timer = setTimeout(() => current.child.kill(), 1500);
    timer.unref?.();
    await current.closed;
    clearTimeout(timer);
  }

  async function handle(payload, signal) {
    if (disposed || signal?.aborted) throw new Error('桌面连接已关闭。');
    if (!payload || typeof payload !== 'object' || Array.isArray(payload)
      || !['status', 'detach', 'close'].includes(payload.action)
      || Object.keys(payload).some(key => !['action', 'character', 'size', 'grabX', 'grabY'].includes(key))) {
      throw new Error('无效的桌面挂件请求。');
    }
    if (payload.action === 'close') {
      await stop();
      return { available: true, active: false };
    }
    const executable = await resolveExecutable();
    if (disposed || signal?.aborted) throw new Error('桌面连接已关闭。');
    if (payload.action === 'status') return { available: !!executable, active: !!session && !session.closing };
    if (!executable) throw new Error('尚未安装桌面组件，请按插件说明完成桌面设置。');
    if (!characterIds.has(payload.character) || !Number.isInteger(payload.size)
      || payload.size < 100 || payload.size > 220
      || ![payload.grabX, payload.grabY].every(value => Number.isFinite(value) && value >= 0 && value <= 1)) {
      throw new Error('角色、大小或拖动位置无效。');
    }
    if (session) {
      if (session.closing) throw new Error('桌面挂件正在关闭，请稍后重试。');
      await session.ready;
      return { available: true, active: true };
    }
    const child = spawnProcess(executable, [
      '--detached', payload.character, String(payload.size), '--grab', String(payload.grabX), String(payload.grabY),
    ], { stdio: ['pipe', 'pipe', 'ignore'], windowsHide: true, shell: false });
    let resolveReady;
    let rejectReady;
    let resolveClosed;
    const current = {
      child, closing: false,
      ready: new Promise((resolve, reject) => { resolveReady = resolve; rejectReady = reject; }),
      closed: new Promise(resolve => { resolveClosed = resolve; }),
    };
    session = current;
    let output = '';
    const timeout = setTimeout(() => {
      rejectReady(new Error('桌面挂件启动超时。'));
      child.kill();
    }, 10_000);
    const cancelled = () => { rejectReady(new Error('已取消拖出。')); child.kill(); };
    signal?.addEventListener('abort', cancelled, { once: true });
    child.stdout.on('data', chunk => {
      output += chunk.toString();
      if (output.length > 256) {
        rejectReady(new Error('桌面挂件返回了无效响应。'));
        child.kill();
      } else if (output.includes('\n')) {
        if (output.trim() === '{"event":"ready"}') resolveReady();
        else { rejectReady(new Error('桌面挂件版本不兼容。')); child.kill(); }
      }
    });
    child.stdin.on('error', () => { /* A closed native window may close its pipe first. */ });
    child.on('error', () => rejectReady(new Error('无法启动桌面挂件。')));
    child.once('close', () => {
      clearTimeout(timeout);
      signal?.removeEventListener('abort', cancelled);
      rejectReady(new Error('桌面挂件已关闭。'));
      if (session === current) session = null;
      resolveClosed();
    });
    try {
      await current.ready;
      if (disposed || signal?.aborted || current.closing || session !== current) throw new Error('已取消拖出。');
      return { available: true, active: true };
    } finally {
      clearTimeout(timeout);
      signal?.removeEventListener('abort', cancelled);
    }
  }

  return {
    handle,
    async dispose() { disposed = true; await stop(); },
  };
}

export async function desktopResponse(host, request) {
  if (request.headers.get('content-type')?.split(';')[0].trim() !== 'application/json') {
    return new Response('Expected JSON', { status: 415 });
  }
  let envelope;
  try {
    const reader = request.body?.getReader();
    if (!reader) return new Response('Missing body', { status: 400 });
    const parts = [];
    let size = 0;
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > 4096) { await reader.cancel(); return new Response('Request too large', { status: 413 }); }
      parts.push(value);
    }
    envelope = JSON.parse(Buffer.concat(parts).toString('utf8'));
  } catch { return new Response('Invalid JSON', { status: 400 }); }
  if (envelope?.type !== 'client-request' || envelope.method !== endpoint
    || typeof envelope.rpcId !== 'string' || envelope.rpcId.length > 128) {
    return new Response('Invalid request', { status: 400 });
  }
  let result;
  try { result = { ok: true, value: await host.handle(envelope.payload, request.signal) }; }
  catch (error) {
    result = { ok: false, error: { code: 'bongodsh/desktop-unavailable', message: error.message, details: {} } };
  }
  return Response.json({ type: 'server-response', rpcId: envelope.rpcId, result }, {
    headers: { 'cache-control': 'no-store' },
  });
}
