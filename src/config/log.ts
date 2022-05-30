import { createLogger, format, transports, LogEntry } from 'winston';
import { MODE_ENV_DEVELOPMENT, MODE_ENV_TEST, LOCATION_LOG_COMBINED, LOCATION_LOG_ERROR, SHOW_LOGS } from './env';
import { TLoggerColors, TLogLevels } from '../types';

const { errors, combine, timestamp, splat }	= format;

const consoleLogFormatter = (info: LogEntry): string => {
	const level = info.level as TLogLevels;
	const bgColor: TLoggerColors = {
		debug: `\x1b[42m`,
		error: `\x1b[41m`,
		verbose: `\x1b[46m`,
		warn: `\x1b[43m`
	};
	const fgColor: TLoggerColors = {
		debug: `\x1b[32m`,
		error: `\x1b[31m`,
		verbose: `\x1b[36m`,
		warn: `\x1b[33m`
	};
	const bgBlack = `\x1b[40m`;
	const fgWhite = `\x1b[37m`;
	const fgBlack = `\x1b[30m`;
	let formattedString = `${fgBlack}${bgColor[level]}${info.level.toUpperCase().padEnd(7, ' ')}${bgBlack}${fgColor[level]}${info.timestamp.substring(10, 23)} `;
	if (info.log) formattedString += `\n${JSON.stringify(info.log)}`;
	formattedString += info.stack ? `${info.stack}` : `${ JSON.stringify(info.message)}` ;
	formattedString += info.uuid ? `\n${info.uuid}` : '';
	formattedString += fgWhite;
	return formattedString;
};

export const log = createLogger({
	level: 'debug',
	format: combine(
		timestamp(),
		errors({ stack: true }),
		splat(),
		format.json()
	),
	transports: [
		new transports.File({ filename: LOCATION_LOG_ERROR, level: 'error' }),
		new transports.File({ filename: LOCATION_LOG_COMBINED }),
	],
	exitOnError: false,
});

if (MODE_ENV_DEVELOPMENT || SHOW_LOGS) {
	log.add(
		new transports.Console({
			handleExceptions: true,
			level: 'debug',
			format: combine(
				timestamp({ format: 'YYYY-MM-DD HH:mm:ss.SSS' }),
				errors({ stack: true }),
				splat(),
				format.printf((info) => consoleLogFormatter(info))
			),
		})
	);
}
if (MODE_ENV_TEST) log.removeAllListeners();