export interface Price {
    amount: number,
    currency: string,
    level: PriceLevel
}

export enum PriceLevel {
    CHEAP = 'CHEAP',
    NORMAL = 'NORMAL',
    EXPENSIVE = 'EXPENSIVE',
}
