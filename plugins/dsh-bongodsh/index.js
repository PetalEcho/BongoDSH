import { createDesktopHost, desktopResponse } from './desktop-host.js';

export default {
  name: 'dsh-bongodsh',
  apply(ctx) {
    ctx.inject(['connection'], connectionCtx => {
      const host = createDesktopHost();
      connectionCtx.effect(() => () => host.dispose());
      // Connection owns admission and rejects non-owner peers for exact Fetch routes.
      connectionCtx.connection.fetch.register({
        path: '/api/bongodsh/desktop', methods: ['POST'], requestBody: 'streaming',
        fetch: request => desktopResponse(host, request),
      });
    });
  },
};
