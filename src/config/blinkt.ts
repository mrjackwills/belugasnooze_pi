import { BinaryValue, Gpio } from 'onoff';
import { TLED, TPixel, TPixels } from '../types';

export class Blinkt {
	static #instance: Blinkt;

	static getInstance (clearOnExit: boolean): Blinkt {
		this.#instance = Blinkt.#instance ? Blinkt.#instance : new Blinkt(clearOnExit);
		return this.#instance;
	}

	#clk: Gpio;
	#dat: Gpio;
	#pixels: TPixels;
	#num_pixels: number;
	#clear_on_exit = false;

	private constructor (clearOnExit: boolean) {
		this.#clk = new Gpio(24, 'out');
		this.#dat = new Gpio(23, 'out');
		this.#num_pixels = 8;
		this.#pixels = [];
		for (const _i of Array(this.#num_pixels)) this.#pixels.push({ r: 0, g: 0, b: 0, brightness: 0 });
		if (clearOnExit) this.#setClearOnExit();
	}

	#cleanup (): void {
		this.clear();
		process.exit();
	}

	#eof (): void {
		this.#writeDataNTimes(0, 36);
	}

	#getBrightness (brightness: number): number {
		return 31 * brightness & 0b11111;
	}

	#getPixel ({ r = 0, g = 0, b = 0, brightness }:TLED): TLED {
		return { r: r & 255, g: g & 255, b: b & 255, brightness: this.#getBrightness(brightness) };
	}

	#setClearOnExit (): void {
		if (this.#clear_on_exit) return;
		this.#clear_on_exit = true;
		process.on('exit', () => this.#cleanup());
		process.on('SIGINT', () => this.#cleanup());
	}

	#sof (): void {
		this.#writeDataNTimes(0, 32);
	}

	#writeByte (byte: number): void {
		for (const [ index, _pixel ] of this.#pixels.entries()) {
			const bit = (byte & 1 << 7 - index) > 0 === true ? Gpio.HIGH : Gpio.LOW;
			this.#writeData(bit);
		}
	}

	#writeData (bit: BinaryValue): void {
		this.#dat.writeSync(bit);
		this.#clk.writeSync(1);
		this.#clk.writeSync(0);
	}

	#writeDataNTimes (bit: BinaryValue, cycles: number): void {
		for (const _i of new Array(cycles)) this.#writeData(bit);
	}

	#writePixel (pixel: TLED): void {
		const { r, g, b, brightness } = pixel;
		// brightness is just 0-31?
		this.#writeByte(0b11100000 | brightness);
		this.#writeByte(b);
		this.#writeByte(g);
		this.#writeByte(r);
	}

	clear (): void {
		this.setAll({ r: 0, g: 0, b: 0, brightness: 0 });
		this.show();
	}

	setAll ({ r, g, b, brightness }: TLED): void {
		for (const [ index, _item ] of this.#pixels.entries()) this.setPixel({ index, r, g, b, brightness });
	}
	
	setPixel ({ index, r, g, b, brightness }: TPixel): void {
		this.#pixels[index] = this.#getPixel({ r, g, b, brightness });
	}

	show (): void {
		this.#sof();
		for (const pixel of this.#pixels) this.#writePixel(pixel);
		this.#eof();
	}
}