import { addWifi, piStatus, quit, setTimeZone } from './lib/piSettings.js';
import { alarmController } from './lib/alarm.js';
import { Controls } from './lib/controls';
import { log } from './config/log.js';
import { parseMessage } from './lib/helpers.js';
import { reconnectingWebSocket, TReconnectingWebSocket } from './lib/websocket';
import { TAlarmId, TVueAlarm, TWifi } from './types';

class Handler {

	#ws!: TReconnectingWebSocket;
	#blinkt!: Controls;

	#init (): void {
		try {
			alarmController.selectAndSchedule();
			this.#openListeners();
		} catch (e) {
			log.error('init error');
			quit();
		}
	}

	#openListeners (): void {

		this.#ws.bus.on('ledStatus', (data: boolean) => {
			this.#ws.sendToServer({ data: { name: 'ledStatus', data } });
		});

		this.#ws.bus.on('wsOpen', async () => {
			const currentHour = new Date().getHours();
			if (currentHour > 7 && currentHour < 22) this.#blinkt.rainbowLights();
			this.#sendStatusAndAlarms();
		});

		this.#ws.bus.on('wsClose', async () => {
			log.verbose('websocket connection closed');
		});

		this.#ws.bus.on('wsMessage', (message: string) => {
			try {
				const data = parseMessage(message);
				if (data?.data) {
					const { name, body } = data.data;
					if (body) {
						switch (name) {
						case 'addAlarm':
							// TODO typechecking here!
							// either custom type guards, or joi
							alarmController.addAlarm(<TVueAlarm>body);
							this.#sendStatusAndAlarms();
							break;
						case 'deleteOne':
							alarmController.deleteOne(<TAlarmId>body);
							this.#sendStatusAndAlarms();
							break;
						case 'timeZone':
							setTimeZone(<string>body);
							break;
						case 'wifi':
							addWifi(<TWifi>body);
							break;
						}
		
					} else {
						switch (name) {
						case 'ledStatus':
							this.#ws.sendToServer({ data: { name: 'ledStatus', data: this.#blinkt.ledStatus } });
							break;
						case 'deleteAll':
							alarmController.deleteAll();
							this.#sendStatusAndAlarms();
							break;
						case 'lightoff':
							this.#blinkt.turnOff();
							break;
						case 'lighton':
							this.#blinkt.turnOn();
							break;
						case 'restart':
							quit();
							break;
						case 'status': {
							this.#sendStatusAndAlarms();
							break;
						}
						}
					}
				}
			} catch (e) {
				log.error(e);
			}
		});
	}

	async #sendStatusAndAlarms (): Promise<void> {
		const status = await piStatus();
		const alarms = alarmController.selectAll();
		this.#ws.sendToServer({ data: { name: 'status', data: { ...status, alarms } }, cache: true });
	}

	constructor (ws: TReconnectingWebSocket, blinkt: Controls) {
		this.#ws = ws;
		this.#blinkt = blinkt;
		this.#init();
	}
}

new Handler(reconnectingWebSocket, Controls.getInstance());