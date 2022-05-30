import { config } from 'dotenv';
import { resolve } from 'path';
import { api_version } from './api_version';

config({ path: resolve(__dirname, '../../.env') });

const major = api_version.split('.')[0];
const env = process.env;

if (isNaN(Number(major))) throw new Error('!env major');

if (!env.LOCATION_LOG_COMBINED) throw new Error('!env.LOCATION_LOG_COMBINED');
if (!env.LOCATION_LOG_ERROR) throw new Error('!env.LOCATION_LOG_ERROR');
if (!env.LOCATION_SQLITE) throw new Error('!env.LOCATION_SQLITE');
if (!env.LOCATION_IP_ADDRESS) throw new Error('!env.LOCATION_IP_ADDRESS');

if (!env.WS_ADDRESS) throw new Error('!env.WS_ADDRESS');
if (!env.WS_APIKEY) throw new Error('!env.WS_APIKEY');
if (!env.WS_PASSWORD) throw new Error('!env.WS_PASSWORD');
if (!env.WS_AUTH_ADDRESS) throw new Error('!env.WS_AUTH_ADDRESS');

export const LOCATION_LOG_COMBINED = env.LOCATION_LOG_COMBINED;
export const LOCATION_LOG_ERROR = env.LOCATION_LOG_ERROR;

export const LOCATION_SQLITE = env.LOCATION_SQLITE;

export const SHOW_LOGS = env.SHOW_LOGS;

export const WS_ADDRESS = env.WS_ADDRESS;
export const WS_APIKEY = env.WS_APIKEY;
export const WS_PASSWORD = env.WS_PASSWORD;
export const WS_AUTH_ADDRESS = env.WS_AUTH_ADDRESS;

export const LOCATION_IP_ADDRESS = env.LOCATION_IP_ADDRESS;

export const MODE_ENV_DEVELOPMENT = env.NODE_ENV === 'development';
export const MODE_ENV_PRODUCTION = env.NODE_ENV === 'production';
export const MODE_ENV_TEST = env.NODE_ENV === 'test';
export const API_MAJOR = Number(major);
