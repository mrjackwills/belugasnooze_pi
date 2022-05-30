import { parse } from 'secure-json-parse';
import { TWSfromServer } from '../types';

export const sleep = async (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

export const parseMessage = (message: string): TWSfromServer | undefined => {
	try {
		const tmpData: TWSfromServer = parse(message, undefined, { protoAction: 'remove', constructorAction: 'remove' });
		if (!tmpData.data && !tmpData.error) throw Error('Invalid data');
		return tmpData;
	} catch (e) {
		return undefined;
	}
};