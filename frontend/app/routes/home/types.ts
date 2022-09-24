export type Weekday = 'MON' | 'TUE' | 'WED' | 'THU' | 'FRI' | 'SAT' | 'SUN'
export type PriceLevel = 'CHEAP' | 'NORMAL' | 'EXPENSIVE'
export type NaiveTime = `${number}:${number}`

export interface Schedule {
    id: string;
    level: PriceLevel;
    days: Weekday[];
    hours: {
        from: NaiveTime;
        to: NaiveTime;
    }[],
}