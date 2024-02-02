import {differenceInSeconds, formatISO} from 'date-fns';

/**
 * Determines if two datetimes are within a specified number of seconds from each other.
 * @param date1 The first date to compare. Can be a `Date` object or a string that can be parsed into a `Date`.
 * @param date2 The second date to compare. Can be a `Date` object or a string that can be parsed into a `Date`.
 * @param seconds The number of seconds within which the two dates should be to return `true`.
 * @returns `true` if the difference between the dates is less than or equal to the specified `seconds`, otherwise `false`.
 */
export function isWithinIntervalInSeconds(date1: Date | string, date2: Date | string, seconds: number): boolean {
    const parsedDate1 = typeof date1 === 'string' ? new Date(date1) : date1;
    const parsedDate2 = typeof date2 === 'string' ? new Date(date2) : date2;
    return Math.abs(differenceInSeconds(parsedDate1, parsedDate2)) <= seconds;
}

const timeZone = process.env.TIMEZONE || 'Europe/Oslo';
export function now(): string {
    return formatISO(new Date().toLocaleString('en-US', { timeZone }));
}
