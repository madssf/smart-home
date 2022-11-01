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

export interface Consumption {
    label: string,
    kwh: number,
}

export interface LiveConsumption {
    timestamp: string,
    power: number,
}

export interface RoomTemp {
    room_name: string,
    temp: number
}
