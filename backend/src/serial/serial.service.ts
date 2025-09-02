import { Injectable, OnModuleDestroy, OnModuleInit } from '@nestjs/common';
import { EventEmitter } from 'events';
// serialport has ESM/CJS shape differences and limited typings; require at runtime and keep as any
const SP: any = require('serialport');
const ReadlineParserModule: any = require('@serialport/parser-readline');
// normalize constructor and parser
const SerialCtor: any = SP?.SerialPort ?? SP;
const ParserCtor: any = ReadlineParserModule?.ReadlineParser ?? ReadlineParserModule;

type PortInfo = {
  path: string;
  manufacturer?: string;
  serialNumber?: string;
  pnpId?: string;
  locationId?: string;
  productId?: string;
  vendorId?: string;
};

@Injectable()
export class SerialService implements OnModuleDestroy, OnModuleInit {
  private emitter = new EventEmitter();
  private port: any = null;
  private parser: any = null;
  private verbose = true; // internal debug; does not change emitted payload format

  constructor() {
    // Wrap emitter.emit to log every emitted event centrally
    const origEmit = this.emitter.emit.bind(this.emitter);
    this.emitter.emit = (event: string, ...args: any[]) => {
      if (this.verbose) {
        try {
          const printable = args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a));
          console.log('[SerialService][EVENT]', event, printable.join(' '));
        } catch {
          console.log('[SerialService][EVENT]', event, args.length, 'arg(s)');
        }
      }
      return origEmit(event, ...args);
    };
  }

  async listPorts(): Promise<PortInfo[]> {
  console.log('[SerialService] listPorts()');
  let ports: any[] = [];
    try {
      if (typeof SP.list === 'function') {
        ports = await SP.list();
      } else {
        // try the separate list package
        try {
          const listMod: any = require('@serialport/list');
          if (typeof listMod.list === 'function') ports = await listMod.list();
          else if (typeof listMod === 'function') ports = await listMod();
        } catch (e) {
          // fallback: empty list
          ports = [];
        }
      }
    } catch (e) {
      console.error('[SerialService] listPorts error', e);
      ports = [];
    }
    console.log('[SerialService] listPorts ->', ports);
    return ports.map((p: any) => ({
      path: p.path || p.comName,
      manufacturer: p.manufacturer,
      serialNumber: p.serialNumber,
      pnpId: p.pnpId,
      locationId: p.locationId,
      productId: p.productId,
      vendorId: p.vendorId,
    }));
  }

  open(path: string, baudRate = 115200) {
    this.close();
    console.log('[SerialService] opening port', path, 'baud', baudRate);
    this.port = new SerialCtor({ path, baudRate, autoOpen: false } as any);
    this.parser = this.port.pipe(new ParserCtor({ delimiter: '\n' }));

    this.port.on('open', () => {
      console.log('[SerialService] port open', path);
      this.emitter.emit('open');
    });
    this.port.on('error', (err: any) => {
      console.error('[SerialService] port error', err);
      this.emitter.emit('error', err?.message ?? String(err));
    });
  this.port.on('close', () => { this.emitter.emit('close'); });
    this.parser.on('data', (line: string) => {
      const clean = line.replace(/[\r\n]+$/, '');
      if (this.verbose) console.log('[SerialService] RX:', clean);
      this.emitter.emit('data', clean);
    });

    // open the port
    try {
      this.port.open((err: any) => {
        if (err) {
          console.error('[SerialService] open callback error', err);
          this.emitter.emit('error', err?.message ?? String(err));
        } else {
          console.log('[SerialService] open callback success');
        }
      });
    } catch (e: any) {
      console.error('[SerialService] open exception', e);
      this.emitter.emit('error', e?.message ?? String(e));
    }
  }

  close() {
    if (this.parser) {
      this.parser.removeAllListeners();
      this.parser = null;
    }
    if (this.port) {
      try {
  console.log('[SerialService] closing port', this.port?.path ?? '(unknown)');
  this.port.close();
      } catch (e) {
  console.error('[SerialService] close error', e);
      }
      this.port = null;
    }
    this.emitter.emit('close');
  }

  send(line: string) {
    if (!this.port || !(this.port.writable || this.port.isOpen)) {
      console.warn('[SerialService] send called but port not open');
      throw new Error('Port not open');
    }
    // write and flush
    console.log('[SerialService] TX:', line);
    this.port.write(line + '\n', (err: any) => {
      if (err) console.error('[SerialService] write error', err);
      else console.log('[SerialService] write success');
    });
  }

  status() {
    const s = {
      open: !!(this.port && (this.port.isOpen || this.port.writable)),
      path: this.port?.path ?? null,
    };
    console.log('[SerialService] status ->', s);
    return s;
  }

  on(event: 'data' | 'open' | 'close' | 'error', cb: (...args: any[]) => void) {
    this.emitter.on(event, cb);
    return () => this.emitter.off(event, cb);
  }
  // Allow subscription to extended events (disconnect, drain, etc.)
  onAny(event: string, cb: (...args: any[]) => void) {
    this.emitter.on(event, cb);
    return () => this.emitter.off(event, cb);
  }

  onModuleDestroy() {
    this.close();
  }

  async onModuleInit() {
    // Priority: explicit env vars > auto-detect
    const explicit = process.env.SERIAL_PORT || process.env.PICO_SERIAL_PORT;
    const baud = Number(process.env.SERIAL_BAUD || process.env.PICO_BAUD || 115200);
    if (explicit) {
      try {
        console.log('[SerialService] AUTO opening explicit port', explicit, 'baud=', baud);
        this.open(explicit, baud);
        return;
      } catch (e: any) {
        console.warn('[SerialService] explicit open failed', e?.message ?? e);
      }
    }

    // Attempt auto-detect of Pico (USB first, then common UART paths)
    try {
      const ports = await this.listPorts();
      // Prefer vendorId 2e8a (Pico)
      let candidate = ports.find(p => (p.vendorId || '').toLowerCase() === '2e8a');
      if (!candidate) {
        const common = ['/dev/serial0', '/dev/ttyACM0', '/dev/ttyAMA0'];
        candidate = ports.find(p => p.path && common.includes(p.path));
      }
      // Windows fallback: highest numbered COM port
      if (!candidate) {
        const winCom = ports
          .filter(p => /^COM\d+$/i.test(p.path))
          .sort((a, b) => Number(b.path.replace(/\D/g, '')) - Number(a.path.replace(/\D/g, '')))[0];
        if (winCom) candidate = winCom;
      }
      if (candidate && candidate.path) {
        console.log('[SerialService] AUTO opening detected port', candidate.path);
        this.open(candidate.path, baud);
      } else {
        console.log('[SerialService] No serial port auto-detected (will remain idle)');
      }
    } catch (e: any) {
      console.warn('[SerialService] auto-detect failed', e?.message ?? e);
    }
  }
}
