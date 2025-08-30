# Pico Serial Motor Control — README for Copilot/Developers

Purpose
-------
This file documents the exact serial message contract between the Raspberry Pi Pico firmware and a TypeScript backend. It's written so another Copilot or developer can implement a backend and extend firmware features without guesswork.

Transport modes
---------------
The firmware supports two serial transports (selected automatically):

1. USB CDC (preferred)
	 - When the Pico is connected over USB and a host opens the port, firmware uses `Serial`.
	 - Typical device name:
		 - Windows: `COMx`
		 - Linux (Pi): `/dev/ttyACM0`
	 - Vendor/Product IDs: 0x2E8A / 0x000A (can be used for auto-detection).

2. UART0 on GPIO0/GP0 (TX) and GPIO1/GP1 (RX)
	 - Activated if USB CDC is not opened within ~2 seconds at boot; firmware falls back to `Serial1`.
	 - Used for headless integration when the Pico is cabled to a Raspberry Pi's UART pins.

Wiring (Pico UART0 <-> Raspberry Pi UART)
----------------------------------------
All signals are 3.3V. DO NOT connect 5V to Pico GPIOs.

| Function | Pico Pin | Pi (BCM) | Pi Physical Pin |
|----------|----------|----------|-----------------|
| UART0 TX | GP0      | RXD0 (15)| 10              |
| UART0 RX | GP1      | TXD0 (14)| 8               |
| GND      | GND      | GND      | (any GND)       |

Power options:
- Preferred: Power Pico over USB (isolated data + power). Only connect GND and TX/RX for UART logic level link.
- Alternate: Power from Pi 3V3 pin to Pico 3V3 pin (NOT VBUS) plus GND. Never power from USB and Pi 3V3 simultaneously unless you know backfeed protection is in place.

Raspberry Pi configuration (enable UART)
---------------------------------------
Edit `/boot/firmware/config.txt` (newer Raspberry Pi OS) or `/boot/config.txt` (older) and ensure:

```
enable_uart=1
```

If the serial console/login is enabled, you may need to disable it (via `sudo raspi-config` -> Interface Options -> Serial) so `/dev/serial0` is free.

After reboot you should see one of:
- `/dev/serial0` (symlink to the primary UART)
- `/dev/ttyAMA0` or `/dev/ttyS0` depending on Pi model.

Docker usage on Raspberry Pi
----------------------------
Expose the UART device inside the container:

```
docker run \
	--device /dev/serial0:/dev/serial0 \
	-e PICO_SERIAL_PORT=/dev/serial0 \
	your-image:tag
```

If using USB instead of GPIO UART, expose `/dev/ttyACM0` (or appropriate) similarly.

Single combined app container (frontend + backend)
-------------------------------------------------
The project Dockerfile now builds both the frontend and backend. To run on a Pi with GPIO UART wiring:

```
docker build -t schafkop-app .
docker run --rm \
	--device /dev/serial0:/dev/serial0 \
	-e PICO_SERIAL_PORT=/dev/serial0 \
	-p 80:3000 \
	schafkop-app
```

For USB Pico:
```
docker run --rm \
	--device /dev/ttyACM0:/dev/ttyACM0 \
	-e PICO_SERIAL_PORT=/dev/ttyACM0 \
	-p 80:3000 \
	schafkop-app
```

If you omit PICO_SERIAL_PORT the backend will attempt auto-detection (vendorId 2e8a or common paths).

Environment variables (suggested backend behavior):
- `PICO_SERIAL_PORT`: If set, backend uses this path directly.
- `PICO_BAUD`: Override baud rate (default 115200).

Backend port auto-detection logic (recommended order):
1. If `PICO_SERIAL_PORT` env var exists, use it.
2. Else list serial ports; prefer any with vendorId `2e8a` (Pico USB).
3. Else probe common paths: `/dev/serial0`, `/dev/ttyACM0`, `/dev/ttyAMA0`, Windows `COM` ports (highest matching newly added), macOS `/dev/tty.usbmodem*`.
4. Fallback: error with clear message.

Contract (high-level)
---------------------
- Commands to Pico: ASCII text lines terminated by LF ("\\n") or CRLF ("\\r\\n").
- Events/logs from Pico: ASCII text lines, one event per line. Backend must split on newlines.
- Baud rate: 115200 (firmware uses Serial.begin(115200)).

Accepted commands (input to Pico)
---------------------------------
1) STEP <steps> <direction>
	 - steps: integer (positive number of micro-steps)
	 - direction: 1 (forward) or 0 (reverse)
	 - Example: `STEP 4096 1`

2) SPEED <delay_us>
	 - delay_us: integer microseconds between internal step micro-operations
	 - Example: `SPEED 3000`

Notes:
- Commands are trimmed for leading/trailing whitespace before parsing.
- Invalid or malformed commands result in an error event emitted by the Pico.

Events/log lines emitted by Pico (output)
----------------------------------------
The firmware emits structured lines with prefixes so the backend can parse them easily.

- `EVENT:RECEIVED <raw-command>`
	- Emitted immediately when a command line is received (before execution).
	- Example: `EVENT:RECEIVED STEP 4096 1`

- `EVENT:COMPLETED STEP <steps> <direction>`
	- Emitted after a successful STEP command completes.
	- Example: `EVENT:COMPLETED STEP 4096 1`

- `EVENT:COMPLETED SPEED <delay_us>`
	- Emitted after the SPEED command is applied.

- `EVENT:COMPLETED ERROR: <message>`
	- Emitted when a command fails to parse or run.
	- Example: `EVENT:COMPLETED ERROR: malformed STEP command`

- `STATUS: OK` or `STATUS: ERROR: <message>`
	- Short summary always emitted after processing a command.

Parsing guidance for backend:
- Read raw bytes, split on `\\n` (handle `\\r\\n`).
- Ignore empty lines.
- Inspect prefixes `EVENT:RECEIVED`, `EVENT:COMPLETED`, `STATUS:` and parse the remainder.

Backend responsibilities (TypeScript)
------------------------------------
- Open the serial port at 115200 baud.
- Provide a small API:
	- `sendStep(steps: number, direction: 0|1): Promise<void>`
	- `setSpeed(delayUs: number): Promise<void>`
	- `onEvent(cb: (evt: {type: string; payload: string}) => void)`
- Send commands as ASCII lines terminated by `\\n`.
- Listen for events; map them to higher-level promises if desired.
- Implement reconnect logic with exponential backoff if the device disconnects.
- Add basic validation before sending commands to avoid malformed requests.

Suggested libraries / implementation notes
-----------------------------------------
- Use `serialport` (npm) and its `ReadlineParser` or equivalent.
- Keep a small FIFO of pending commands if you want to wait for `EVENT:COMPLETED` per command.
- For each command you can:
	1. Write the line `CMD\\n` to the port.
	2. Wait for `EVENT:RECEIVED CMD` then `EVENT:COMPLETED ...` and `STATUS: ...`.
- Ensure the backend tolerates duplicate or out-of-order messages (don't assume perfect timing).

Examples
--------
Example sequence for a STEP command:

```
// backend writes:
STEP 4096 1

// pico emits:
EVENT:RECEIVED STEP 4096 1
EVENT:COMPLETED STEP 4096 1
STATUS: OK
```

TypeScript example (auto-detect & simple API)
--------------------------------------------
```ts
import { SerialPort } from 'serialport';
import { ReadlineParser } from '@serialport/parser-readline';

interface PicoEvent { type: string; payload: string; raw: string; }

async function findPort(): Promise<string> {
	if (process.env.PICO_SERIAL_PORT) return process.env.PICO_SERIAL_PORT;
	const ports = await SerialPort.list();
	// Prefer Pico USB (vendorId 2e8a)
	const pico = ports.find(p => (p.vendorId||'').toLowerCase()==='2e8a');
	if (pico && pico.path) return pico.path;
	const candidates = ['/dev/serial0','/dev/ttyACM0','/dev/ttyAMA0'];
	for (const c of candidates) if (ports.find(p=>p.path===c)) return c;
	if (ports[0]) return ports[0].path; // fallback
	throw new Error('No serial ports detected for Pico');
}

export class PicoClient {
	private port!: SerialPort;
	private parser!: ReadlineParser;
	private listeners: ((e:PicoEvent)=>void)[] = [];
	constructor(private baud = Number(process.env.PICO_BAUD)||115200) {}
	async init() {
		const path = await findPort();
		this.port = new SerialPort({ path, baudRate: this.baud });
		this.parser = this.port.pipe(new ReadlineParser({ delimiter: '\n' }));
		this.parser.on('data', line => {
			const l = line.trim();
			if (!l) return;
			let type='RAW'; let payload=l;
			if (l.startsWith('EVENT:RECEIVED ')) { type='EVENT:RECEIVED'; payload=l.substring(15); }
			else if (l.startsWith('EVENT:COMPLETED ')) { type='EVENT:COMPLETED'; payload=l.substring(17); }
			else if (l.startsWith('STATUS: ')) { type='STATUS'; payload=l.substring(8); }
			this.listeners.forEach(cb=>cb({ type, payload, raw:l }));
		});
		return this;
	}
	onEvent(cb:(e:PicoEvent)=>void){ this.listeners.push(cb); }
	private write(cmd:string){ this.port.write(cmd.endsWith('\n')?cmd:cmd+'\n'); }
	sendStep(steps:number, dir:0|1){ this.write(`STEP ${steps} ${dir}`); }
	setSpeed(delayUs:number){ this.write(`SPEED ${delayUs}`); }
}

// Usage example
// (async () => {
//   const pico = await new PicoClient().init();
//   pico.onEvent(e => console.log('EVENT', e));
//   pico.setSpeed(3000);
//   pico.sendStep(4096,1);
// })();
```

Manual testing
--------------
- Build and upload firmware with PlatformIO (pico environment).
- Open a serial terminal at 115200 and send commands.
- Observe `EVENT:` and `STATUS:` lines.

Testing inside Docker (Pi UART example)
--------------------------------------
1. Connect wiring (see table above) and power Pico.
2. Confirm `/dev/serial0` exists on host (`ls -l /dev/serial0`).
3. Run container with device passed through.
4. Inside container: run a small Node script using the TypeScript example (compiled) or `screen /dev/serial0 115200` for manual check.

Troubleshooting
---------------
| Symptom | Likely Cause | Action |
|---------|--------------|--------|
| No output on UART | Pi serial console still enabled | Disable login shell on serial via `raspi-config` |
| Garbled characters | Baud mismatch | Ensure both sides at 115200 |
| Only EVENT:START appears | Backend not sending newline | Ensure commands end with `\n` |
| USB port not found | Missing udev permissions | Add user to `dialout` (Linux) or run with proper permissions |
| Command times out | Long step count | Consider splitting into smaller STEP commands |

Extending the firmware
----------------------
- Add new command parsing in `src/main.cpp`.
- Emit `EVENT:RECEIVED <cmd>` when a command is received.
- Emit `EVENT:COMPLETED <...>` once the command finishes (or `EVENT:COMPLETED ERROR: ...` on failure).
- Update this README with any new event formats.

Files of interest
-----------------
- `src/main.cpp` — serial parser, command dispatch, and event emission.
- `src/ULN2003Stepper.h`, `src/Stepper.h` — stepper implementation.
- `src/schafkopf-bot.cpp` — helper logic; references globals via `extern`.

Notes for a Copilot implementer
------------------------------
- Preserve the exact prefixes (`EVENT:RECEIVED`, `EVENT:COMPLETED`, `STATUS:`).
- Add unit tests for the backend parser to assert the event shapes.
- Keep CLI/manual examples minimal and copyable.
 - When adding new commands, document: syntax, EVENT:COMPLETED form, error cases, sample sequence.
 - Keep the README as the single source of truth for the serial protocol.

---
This README is intended as the definitive reference for implementing the TypeScript backend and for extending the Pico firmware. Follow the message formats exactly to avoid breaking existing parsers.
