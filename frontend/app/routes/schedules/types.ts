import type {PriceLevel} from "~/routes/types";

export const WEEKDAYS = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'] as const;
export type Weekday = typeof WEEKDAYS[number]

export const getWeekdayName = (weekday: Weekday) => {
    switch (weekday) {
        case 'MON':
            return 'Monday';
        case 'TUE':
            return 'Tuesday';
        case 'WED':
            return 'Wednesday';
        case 'THU':
            return 'Thursday';
        case 'FRI':
            return 'Friday';
        case 'SAT':
            return 'Saturday';
        case 'SUN':
            return 'Sunday';
    }
}

export type NaiveTime = `${string}:${string}:${string}`

export type TimeWindow = [NaiveTime, NaiveTime]

export interface Schedule {
    id: string;
    temps: PriceLevelTemps;
    days: Weekday[];
    time_windows: TimeWindow[],
    room_ids: string[],
}

export type PriceLevelTemps = {[key in PriceLevel]?: number}
