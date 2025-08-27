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
    this.parser.on('data', (line: string) => {
      console.log('[SerialService] RX:', line);
      this.emitter.emit('data', line);
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

  onModuleDestroy() {
    this.close();
  }

  onModuleInit() {
    const port = process.env.SERIAL_PORT;
    const baud = process.env.SERIAL_BAUD ? Number(process.env.SERIAL_BAUD) : undefined;
    if (port) {
      try {
        console.log('[SerialService] AUTO opening port from SERIAL_PORT=', port, 'baud=', baud ?? 115200);
        this.open(port, baud ?? 115200);
      } catch (e: any) {
        console.warn('[SerialService] AUTO open failed', e?.message ?? e);
      }
    }
  }
}
