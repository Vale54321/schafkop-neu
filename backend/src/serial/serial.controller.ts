import { Controller, Get, Post, Body, Res, Req } from '@nestjs/common';
import type { Response, Request } from 'express';
import { SerialService } from './serial.service';

@Controller('serial')
export class SerialController {
  constructor(private readonly serial: SerialService) {}

  @Get('ports')
  async ports() {
  console.log('[SerialController] GET /serial/ports');
  const p = await this.serial.listPorts();
  console.log('[SerialController] ports ->', p);
  return p;
  }

  @Post('open')
  async open(@Body() body: { path: string; baud?: number }) {
    console.log('[SerialController] POST /serial/open', body);
    try {
      this.serial.open(body.path, body.baud ?? 115200);
      return { ok: true };
    } catch (e: any) {
      console.error('[SerialController] open error', e);
      return { ok: false, error: String(e) };
    }
  }

  @Post('close')
  async close() {
  console.log('[SerialController] POST /serial/close');
  this.serial.close();
  return { ok: true };
  }

  @Post('send')
  async send(@Body() body: { payload: string }) {
    console.log('[SerialController] POST /serial/send', body.payload);
    try {
      this.serial.send(body.payload);
      return { ok: true };
    } catch (e: any) {
      console.error('[SerialController] send error', e);
      return { ok: false, error: String(e) };
    }
  }

  @Get('status')
  status() {
    return this.serial.status();
  }

  @Get('stream')
  stream(@Req() req: Request, @Res() res: Response) {
    // SSE stream
    res.set({
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      Connection: 'keep-alive',
    });
    res.flushHeaders?.();

    const sendEvent = (event: string, data: any) => {
      res.write(`event: ${event}\n`);
      res.write(`data: ${JSON.stringify(data)}\n\n`);
    };

    const offData = this.serial.on('data', (line) => sendEvent('data', { line }));
    const offOpen = this.serial.on('open', () => sendEvent('open', { ts: Date.now() }));
    const offClose = this.serial.on('close', () => sendEvent('close', { ts: Date.now() }));
    const offErr = this.serial.on('error', (err) => sendEvent('error', { message: String(err) }));

  console.log('[SerialController] SSE client connected');

    // send a ping every 20s to keep connection alive
    const ping = setInterval(() => res.write(': ping\n\n'), 20000);

    req.on('close', () => {
  console.log('[SerialController] SSE client disconnected');
      clearInterval(ping);
      offData();
      offOpen();
      offClose();
      offErr();
    });

    return res;
  }
}
