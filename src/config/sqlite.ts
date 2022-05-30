import sqlite from 'better-sqlite3';
import { LOCATION_SQLITE } from './env';

class Database {
	db!: sqlite.Database;

	constructor () {
		this.db = new sqlite(LOCATION_SQLITE);
		this.db.pragma('journal_mode=WAL');
	}
	
}

export const db = new Database().db;