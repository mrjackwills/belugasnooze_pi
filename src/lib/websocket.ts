import { EventBus } from './events';
import { log } from '../config/log';
import { TCustomEmitter, TWSToSever } from '../types';
import { WS_ADDRESS, WS_APIKEY, WS_AUTH_ADDRESS, WS_PASSWORD } from '../config/env';
import Axios from 'axios';
import ws from 'ws';

class ReconnectingWebSocket {

	#ping_timeout?: NodeJS.Timeout;
	#reconnect_attempts!: number;
	#reconnect_interval!: number;
	#ws_address: string;
	#ws_api_key: string;
	#ws_auth_address: string;
	#ws_password: string;
	#ws!: ws;
	
	constructor () {
		this.#ws_address = WS_ADDRESS;
		this.#ws_api_key = WS_APIKEY;
		this.#ws_auth_address = WS_AUTH_ADDRESS;
		this.#ws_password = WS_PASSWORD;
		this.#resetReconnectionDetails();
		this.#open();
	}

	#checkServer (): void {
		this.#clear();
		this.#ping_timeout = setTimeout(() => this.#ws.terminate(), 75 * 1000);
	}

	#clear (): void {
		if (this.#ping_timeout) clearTimeout(this.#ping_timeout);
	}

	#customEmitter (data: TCustomEmitter): void {
		this.bus.emit(data.emiterName, data.data);
	}
	
	async #getAccessToken (): Promise<string|void> {
		try {
			const authToken = await Axios.post(this.#ws_auth_address, { key: this.#ws_api_key, password: this.#ws_password });
			if (!authToken?.data?.response) throw Error('getAccessToken: !accessCode');
			return authToken.data.response;
		} catch (e) {
			log.debug('access token error');
			log.error(e);
		}
	}
	
	async #open (): Promise<void> {
		try {
			const authToken = await this.#getAccessToken();
			if (!authToken) return this.#reconnect();

			this.#ws = new ws(`${this.#ws_address}/${authToken}`, [ this.#ws_api_key ]);

			this.#ws.on('close', (code: number, data?: Buffer) => {
				if (code !== 1000) this.#reconnect();
				this.#clear();
				const reason = data?.toString();
				const error = reason ? reason : `disconnected @ ${new Date}`;
				log.error(error);
			});

			this.#ws.on('error', (code: number, data?: Buffer) => {
				const reason = data?.toString();
				if (reason === 'ECONNREFUSED') this.#reconnect();
				const error = reason ? reason : `Error code: ${code} @ ${new Date}`;
				log.error(error);
			});

			this.#ws.on('message', (data: Buffer, isBinary: boolean) => {
				if (!isBinary) this.#customEmitter({ data: data.toString(), emiterName: 'wsMessage' });
			}),

			this.#ws.on('open', () => {
				this.#customEmitter({ emiterName: 'wsOpen' });
				this.#resetReconnectionDetails();
			});

			this.#ws.on('ping', () => this.#checkServer());
		
		} catch (e) {
			log.error(e);
		}
	}

	#reconnect (): void {
		this.#clear();
		if (this.#reconnect_attempts === 40) this.#reconnect_interval = 1000 * 60 * 5;
		this.#reconnect_attempts ++;
		this.#ws?.removeAllListeners();
		setTimeout(() => this.#open(), this.#reconnect_interval);
	}

	#resetReconnectionDetails () :void {
		this.#reconnect_interval = 15000;
		this.#reconnect_attempts = 0;
	}
	
	public bus = EventBus.getInstance.bus;
	
	sendToServer (message: TWSToSever): void {
		this.#ws?.send(JSON.stringify(message));
	}
}

export type TReconnectingWebSocket = ReconnectingWebSocket
export const reconnectingWebSocket = new ReconnectingWebSocket();