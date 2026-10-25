// Test-only static server; production has no HTTP server or browser mock adapter.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve, relative, extname } from 'node:path';
import { pnpmSync } from './tool-process.mjs';
const build = pnpmSync(['exec', 'vite', 'build', '--config', 'tests/e2e/vite.config.ts'], {
  stdio: 'inherit',
});
if (build.status !== 0) process.exit(build.status ?? 1);
const root = resolve('target/e2e-site');
const types = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.woff2': 'font/woff2',
};
createServer(async (request, response) => {
  try {
    const path = resolve(
      root,
      `.${decodeURIComponent(new URL(request.url, 'http://localhost').pathname)}`,
    );
    if (relative(root, path).startsWith('..')) throw new Error('Outside test site');
    response.setHeader('Content-Type', types[extname(path)] ?? 'application/octet-stream');
    response.end(await readFile(path));
  } catch {
    response.writeHead(404);
    response.end('Not found');
  }
}).listen(1421, '127.0.0.1');
