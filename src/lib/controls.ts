import { Blinkt } from '../config/blinkt';
import { EventBus } from './events';
import { sleep } from '../lib/helpers';
import { TRGB } from '../types';

export class Controls {
	static #instance: Controls;

	static getInstance (): Controls {
		this.#instance = Controls.#instance ? Controls.#instance : new Controls();
		return this.#instance;
	}

	#blinkt: Blinkt;
	#light_color: TRGB;
	#one_minute = 1000 * 60;
	#sleep_timeout?: NodeJS.Timeout;
	#leds_on: boolean;

	private constructor () {
		this.#leds_on = false;
		this.#light_color = { r: 255, g: 200, b: 15 };
		this.#blinkt = Blinkt.getInstance(true);
	}

	#emitStatus (on:boolean): void {
		this.#leds_on = on;
		this.bus.emit('ledStatus', this.#leds_on);
	}

	get ledStatus (): boolean {
		return this.#leds_on;
	}

	alarmSequence (brightness: number): void {
		if (brightness > 10) return;
		this.#sleep_timeout = brightness === 10 ?
			setTimeout(() => this.turnOff(), this.#one_minute * 45):
			setTimeout(() => this.alarmSequence(brightness+1), this.#one_minute * 5);
		this.#blinkt.setAll({ ...this.#light_color, brightness: brightness / 10 });
		this.#blinkt.show();
		if (brightness === 1) this.#emitStatus(true);
	}

	turnOff (): void {
		if (this.#sleep_timeout) clearTimeout(this.#sleep_timeout);
		this.#blinkt.clear();
		this.#emitStatus(false);
	}

	turnOn (): void {
		if (this.#leds_on) return;
		if (this.#sleep_timeout) clearTimeout(this.#sleep_timeout);
		this.#blinkt.setAll({ ...this.#light_color, brightness: 1 });
		this.#blinkt.show();
		this.#emitStatus(true);
		this.#sleep_timeout = setTimeout(() => {
			this.turnOff();
		}, this.#one_minute * 10);
	}

	async rainbowLights (): Promise<void> {
		if (this.#leds_on) return;
		if (this.#sleep_timeout) clearTimeout(this.#sleep_timeout);
		const rainbow = [
			{ r: 255, g: 0, b: 0 },
			{ r: 255, g: 127, b: 0 },
			{ r: 255, g: 255, b: 0 },
			{ r: 0, g: 255, b: 0 },
			{ r: 0, g: 0, b: 255 },
			{ r: 39, g: 0, b: 51 },
			{ r: 139, g: 0, b: 255 },
			{ r: 255, g: 255, b: 255 },
		];
		for (const [ index, rgb ] of rainbow.entries()) {
			const brightness = (index + 1) / 10;
			this.#blinkt.setPixel({ index, ...rgb, brightness });
			this.#blinkt.show();
			// eslint-disable-next-line no-await-in-loop
			await sleep(30);
			this.#blinkt.clear();
		}
		await sleep(60);
		for (const [ index, rgb ] of rainbow.entries()) {
			const brightness = (10 - index) / 10;
			this.#blinkt.setPixel({ index: 7 - index, ...rgb, brightness });
			this.#blinkt.show();
			// eslint-disable-next-line no-await-in-loop
			await sleep(30);
			this.#blinkt.clear();
		}
	}

	public bus = EventBus.getInstance.bus;
	
}