import { Controls } from './controls';
import { queries } from './queries';
import { scheduleJob, scheduledJobs, RecurrenceRule } from 'node-schedule';
import { TAlarmId, TVueAlarm, TDbAlarm, TDbAlarmZone } from '../types';

class Alarms {

	#blinkt!: Controls;

	constructor (blinkt: Controls) {
		this.#blinkt = blinkt;
	}

	addAlarm ({ days, hour, minute }:TVueAlarm): void {
		const alarm_ids = queries.insert_alarm({ days, hour, minute });
		const zone = queries.select_timezone();
		for (const [ index, a ] of Object(alarm_ids).entries()) {
			const validDay = Number(days[index]);
			if (!isNaN(validDay)) this.scheduleAlarm({ alarm_id: a, day: validDay, hour, minute, zone });
		}
	}

	cancelAll (): void {
		for (const value of Object.values(scheduledJobs)) value.cancel();
	}

	deleteAll (): void {
		queries.delete_all_alarm();
		this.cancelAll();
	}

	deleteOne (data: TAlarmId): void {
		if (!scheduledJobs[Number(data.alarm_id)]) return;
		const job = scheduledJobs[data.alarm_id];
		if (job) job.cancel();
		queries.delete_alarm(data.alarm_id);
	}

	selectAndSchedule (): void {
		for (const value of Object.values(scheduledJobs)) value.cancel();
		const zone = queries.select_timezone();
		const allData = this.selectAll();
		if (allData.length > 0) for (const row of allData) this.scheduleAlarm({ alarm_id: row.alarm_id, day: row.day, hour: row.hour, minute: row.minute, zone });
	}

	scheduleAlarm ({ alarm_id, day, hour, minute, zone }: TDbAlarmZone): void {
		const newAlarm = new RecurrenceRule();
		newAlarm.dayOfWeek = day;
		newAlarm.hour = hour;
		newAlarm.minute = minute;
		newAlarm.tz = zone;
		scheduleJob(`${alarm_id}`, newAlarm, () => this.#blinkt.alarmSequence(1));
	}

	selectAll (): Array<TDbAlarm> {
		return queries.select_allAlarm();
	}
}

export const alarmController = new Alarms(Controls.getInstance());