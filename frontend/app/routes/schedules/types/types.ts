export const WEEKDAYS = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'] as const;
export type Weekday = typeof WEEKDAYS[number]

export const PRICE_LEVELS = ['CHEAP', 'NORMAL', 'EXPENSIVE'] as const;
export type PriceLevel = typeof PRICE_LEVELS[number]
export type NaiveTime = `${string}:${string}`

export type TimeWindow = {
    from: NaiveTime;
    to: NaiveTime;
}

export interface Schedule {
    id: string;
    priceLevel: PriceLevel;
    days: Weekday[];
    hours: TimeWindow[],
}
