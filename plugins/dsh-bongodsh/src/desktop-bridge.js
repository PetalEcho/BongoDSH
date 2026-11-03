// Use DSH's authenticated connection, including its Electron fetch carrier.
export function createDesktopBridge(ctx) {
  let connection = null;
  let pendingDetach = null;
  const lifetime = new AbortController();
  const injection = ctx.inject?.(['connection'], child => {
    connection = child.connection;
    child.effect(() => () => { connection = null; });
  });

  async function request(payload, signal = lifetime.signal) {
    if (!connection?.rpc?.call || connection.isLoopback !== true) {
      throw new Error('拖出功能需要本机 DSH 和已构建的桌面挂件。');
    }
    const result = await connection.rpc.call('/api', 'bongodsh/desktop', payload, signal);
    if (!result.ok) throw new Error(result.error?.message ?? '桌面挂件不可用。');
    return result.value;
  }

  return {
    status: () => request({ action: 'status' }),
    async detach(character, size, grabX, grabY, paused = false) {
      const controller = new AbortController();
      pendingDetach = controller;
      try {
        return await request({ action: 'detach', character, size, grabX, grabY, paused },
          AbortSignal.any([lifetime.signal, controller.signal]));
      } finally { if (pendingDetach === controller) pendingDetach = null; }
    },
    cancelDetach() { pendingDetach?.abort(); },
    close: () => request({ action: 'close' }),
    dispose() { lifetime.abort(); injection?.dispose?.(); },
  };
}
