type Branded<K, T> = K & { __brand: T }
export type TimeZone = Branded<string, 'TimeZone'>

type TBaseNames = 'api_version' | 'ledStatus' | 'status';
type TFromServerNames = TBaseNames | 'addAlarm' | 'deleteAll' | 'deleteOne' | 'lightoff' | 'lighton' | 'restart' | 'timeZone' | 'wifi';

export type TAlarm = THourMinute & TDayId
export type TAlarmId = { alarm_id: string }
export type TAllAlarms = Array<TDbAlarm>;
export type TDayId = TAlarmId & { day: number }
export type TDbAlarm = TAlarm & TAlarmId
export type TDbAlarmZone = TDbAlarm & {zone: TimeZone}
// Think I should set this as string
export type THourMinute = { [ K in 'hour' | 'minute'] : number }
export type TSingleAlarm = THourMinute & {day: number}
export type TInsertAlarm = Array<TAlarmId>
export type TLED = TRGB & { brightness: number }
export type TLEDStatus = { status: boolean }
export type TPixel = TLED & { index: number }
export type TPixels = Array<TLED>;
export type TRGB = { [ K in 'r' | 'g' | 'b']: number }
export type TVueAlarm = THourMinute & {	days: Array<number> }
export type TWifi = { [ K in 'ssid' | 'password']: string }
export type TLogLevels = 'debug' | 'error' | 'verbose' | 'warn'
export type TLoggerColors = { readonly [index in TLogLevels]: string };

export type TValidPixel = {
	type: string;
	valid: boolean;
}

export type TPiStatus = {[ K in 'internalIp' | 'piTime' | 'piVersion' ] : string }
	& {[ K in 'piUptime' | 'piNodeUptime' ] :number }

export type TStatusAndAlarms = TPiStatus & { alarms: TAllAlarms }

export type TWSToSever = {
		data: {name: 'status', data: TStatusAndAlarms} | {name: 'ledStatus', data: boolean}
		cache?: boolean
}

export type TWSfromServer = {
	data?: {name: TFromServerNames, body?: TWifi | TVueAlarm | string | TAlarmId}
	error?: {message: string, code: number}
}

export type TCustomEmitter = {
	data?: string;
	emiterName: 'wsMessage' | 'wsOpen' | 'wsClose'
}