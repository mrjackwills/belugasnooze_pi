import { alarmController } from './alarm';
import { api_version as piVersion } from '../config/api_version';
import { isTimeZone } from '../types/typeguards';
import { LOCATION_IP_ADDRESS } from '../config/env';
import { promises as fs } from 'fs';
import { queries } from './queries';
import { TPiStatus } from '../types';
import { uptime } from 'os';

export const getIp = async (): Promise<string> => {
	const ip_address = await fs.readFile(LOCATION_IP_ADDRESS, 'utf-8');
	return ip_address.trim();
};

export const piStatus = async (): Promise<TPiStatus> => {

	const timeZone = queries.select_timezone();

	const tzOptions: Intl.DateTimeFormatOptions = {
		timeZone,
		hour: 'numeric',
		minute: 'numeric',
		second: 'numeric',
		hour12: false
	};
	const formattedDate = new Intl.DateTimeFormat([], tzOptions);
	
	// xx:xx:xx-region/zone
	const piTime = `${formattedDate.format(new Date())}-${timeZone}`;

	const output: TPiStatus = {
		internalIp: await getIp(),
		piVersion,
		piTime,
		piNodeUptime: Math.trunc(process.uptime()),
		piUptime: Math.trunc(uptime())
	};
	return output;
};

export const quit = (): void => {
	process.exit();
};

export const setTimeZone = (data: string) : void => {
	if (!isTimeZone(data)) return;
	queries.update_timezone(data);
	alarmController.selectAndSchedule();
};