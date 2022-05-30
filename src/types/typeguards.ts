import { TimeZone } from './index';
import { TZ } from '../config/timezones.js';

export const isTimeZone = (input: string): input is TimeZone => TZ.includes(input);