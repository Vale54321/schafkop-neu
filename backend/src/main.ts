import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { NestExpressApplication } from '@nestjs/platform-express';
import { join } from 'path';

async function bootstrap() {
  const app = await NestFactory.create<NestExpressApplication>(AppModule, {
    bufferLogs: true,
  });

  // Serve static frontend (built assets copied to /app/public in container)
  const publicDir = join(process.cwd(), 'public');
  app.useStaticAssets(publicDir);

  // Basic health endpoint
  app.getHttpAdapter().get('/healthz', (_req: any, res: any) => {
    res.json({ ok: true, ts: Date.now() });
  });

  const port = Number(process.env.PORT || 3000);
  await app.listen(port);
  // eslint-disable-next-line no-console
  console.log(`[bootstrap] Listening on :${port}`);
}
bootstrap().catch((e) => {
  // eslint-disable-next-line no-console
  console.error('Fatal bootstrap error', e);
  process.exit(1);
});
