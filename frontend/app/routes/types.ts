import type {Schedule} from "~/routes/schedules/types";

export interface PriceInfo {
    amount: number,
    currency: string,
    ext_price_level: PriceLevel
    price_level: PriceLevel | null
    starts_at: string;
}

export enum PriceLevel {
    VeryCheap = 'VeryCheap',
    Cheap = 'Cheap',
    Normal = 'Normal',
    Expensive = 'Expensive',
    VeryExpensive = 'VeryExpensive',
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
    room_id: string,
    room_name: string,
    temp: number
    time: string,
}

export interface PlugStatus {
    name: string,
    room_id: string,
    is_on: boolean,
    power: number
}

export interface ActiveSchedule {
    room_id: string,
    schedule: Schedule | null,
    temp: number | null,
}
