import { EventEmitter } from 'events';

export class EventBus {

	static #instance: EventBus;

	static get getInstance (): EventBus {
		this.#instance = EventBus.#instance ? EventBus.#instance : new EventBus();
		return this.#instance;
	}

	public bus: EventEmitter;
	
	private constructor () {
		this.bus = new EventEmitter();
	}
}