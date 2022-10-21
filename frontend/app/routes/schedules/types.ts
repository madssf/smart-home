export const WEEKDAYS = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'] as const;
export type Weekday = typeof WEEKDAYS[number]

export const PRICE_LEVELS = ['CHEAP', 'NORMAL', 'EXPENSIVE'] as const;
export type PriceLevel = typeof PRICE_LEVELS[number]
export type NaiveTime = `${string}:${string}:${string}`

export type TimeWindow = [NaiveTime, NaiveTime]

export interface Schedule {
    id: string;
    price_level: PriceLevel;
    days: Weekday[];
    time_windows: TimeWindow[],
    room_ids: string[],
    temp: number,
}
