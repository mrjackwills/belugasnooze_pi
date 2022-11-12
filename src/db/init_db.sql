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
	zone_name TEXT NOT NULL
) STRICT;

COMMIT;