import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';

const files = new Map([
  ['/', [new URL('../preview/index.html', import.meta.url), 'text/html; charset=utf-8']],
  ['/client.js', [new URL('../client.js', import.meta.url), 'text/javascript; charset=utf-8']],
]);
const server = createServer(async (request, response) => {
  const file = files.get(new URL(request.url, 'http://127.0.0.1').pathname);
  if (!file || !['GET', 'HEAD'].includes(request.method)) {
    response.writeHead(404).end();
    return;
  }
  try {
    const content = await readFile(file[0]);
    response.writeHead(200, { 'Content-Type': file[1], 'Cache-Control': 'no-store' });
    response.end(request.method === 'HEAD' ? undefined : content);
  } catch {
    response.writeHead(500).end('Preview file unavailable. Run npm run build.');
  }
});
server.on('error', error => {
  console.error(`无法启动本地预览：${error.message}`);
  process.exitCode = 1;
});
server.listen(0, '127.0.0.1', () => {
  console.log(`BongoDSH preview: http://127.0.0.1:${server.address().port}`);
});
