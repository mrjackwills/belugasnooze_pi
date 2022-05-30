import sqlite from 'better-sqlite3';
import { TAllAlarms, TimeZone, TVueAlarm, TDbAlarm, TSingleAlarm } from '../types';
import { isTimeZone } from '../types/typeguards';
import { db } from '../config/sqlite';

class Queries {
	#db!: sqlite.Database;

	constructor (db: sqlite.Database) {

		this.#db = db;
		this.create_tables();
	}

	create_tables (): void {
		const create_statement = `
BEGIN;

CREATE TABLE IF NOT EXISTS alarm  (
	alarm_id INTEGER PRIMARY KEY AUTOINCREMENT,
	day INTEGER NOT NULL CHECK (day >= 0 AND day <= 6),
	hour INTEGER NOT NULL CHECK (hour >= 0 AND hour <= 23),
	minute INTEGER NOT NULL CHECK (minute >= 0 AND minute <= 59),
	UNIQUE (day, hour, minute)
) STRICT;
		
CREATE TABLE IF NOT EXISTS timezone  (
	timezone_id INTEGER PRIMARY KEY AUTOINCREMENT CHECK (timezone_id = 1),
	zone TEXT NOT NULL DEFAULT 'America/New_York'
) STRICT;

COMMIT;`;
		this.#db.exec(create_statement);
	}

	delete_alarm (alarm_id: string): void {
		if (!alarm_id) throw Error('delete_alarm: !alarm_id');
		if (isNaN(Number(alarm_id))) throw Error('delete_alarm: !isNaN');
		const query = this.#db.prepare('DELETE FROM alarm WHERE alarm_id = ?');
		query.run(alarm_id);
	}

	delete_all_alarm (): void {
		const query = this.#db.prepare('DELETE FROM alarm');
		query.run();
	}

	select_allAlarm (): TAllAlarms {
		const query = this.#db.prepare(`SELECT * FROM alarm`);
		const result = query.all();
		return result;
	}

	select_alarm ({ day, hour, minute }: TSingleAlarm) :TDbAlarm|undefined {
		const query = this.#db.prepare('SELECT * FROM alarm WHERE day = ? AND hour = ? AND minute = ?');
		const data = query.all(day, hour, minute);
		return data[0];
	}

	insert_timezone (zone = 'America/New_York') : void {
		if (!isTimeZone(zone)) throw Error('insert_timeszone: zone invalid');
		const query = this.#db.prepare('INSERT INTO timezone(zone) values(?)');
		query.run(zone);
	}

	update_timezone (zone: TimeZone) : void {
		if (!isTimeZone(zone)) throw Error('update_timezone: zone invalid');
		const query = this.#db.prepare('UPDATE timezone SET zone = ?');
		query.run(zone);
	}

	select_timezone (): TimeZone {
		const query = this.#db.prepare('SELECT zone from timezone');
		const rows = query.all();
		if (rows[0]) {
			const tz = rows[0].zone;
			if (!isTimeZone(tz)) throw Error('select_timezone: zone invalid');
			return tz;
		}
		else {
			this.insert_timezone();
			return this.select_timezone();
		}
	}

	insert_alarm ({ days, hour, minute }: TVueAlarm) : Array<string>|void {

		const conflict = [];
		for (const day of days) {
			const conflicted = this.select_alarm({ day, hour, minute });
			if (conflicted) conflict.push(conflicted);
		}
		
		if (conflict.length > 0) return;

		const insert = this.#db.prepare('INSERT INTO alarm (day, hour, minute) VALUES(?, ?, ?)');

		const output: Array<string>= [];
		const insertMany = this.#db.transaction(() => {
			for (const day of days) {
				const row = insert.run(day, hour, minute);
				output.push(String(row.lastInsertRowid));
			}
		});

		insertMany();
		return output;
	}
}

export const queries = new Queries(db);