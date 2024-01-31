import type {LiveConsumption} from "~/routes/types";

export interface LiveConsumptionData {
    liveConsumption: LiveConsumption[]
    liveConsumptionStats: LiveConsumptionStats
}

export type LiveConsumptionChange = 'UP' | 'NONE' | 'DOWN'
export type LiveConsumptionStats = { consumption: number | null, consumptionChange: LiveConsumptionChange, consumptionTime: string | null }

export const getLiveConsumptionStats = (data: LiveConsumption[]): LiveConsumptionStats => {
    if (data.length === 0) {
        return { consumption: null, consumptionChange: 'NONE', consumptionTime: null };
    } else if (data.length === 1) {
        return { consumption: data[0].power, consumptionChange: 'NONE', consumptionTime: data[0].timestamp };
    } else {
        return {
            consumption: data[0].power,
            consumptionChange: data[0].power === data[1].power ? 'NONE' :
                data[0].power > data[1].power ?
                    'UP' : 'DOWN',
            consumptionTime: data[0].timestamp,
        };
    }
};

export const fromLiveConsumption = (data: LiveConsumption[]): LiveConsumptionData => {
    return {
        liveConsumption: [...data].reverse(),
        liveConsumptionStats: getLiveConsumptionStats([...data]),
    };
}
export const fromSseEvent = (sseEvent: string) => {
    const data: LiveConsumption[] = JSON.parse(sseEvent);
    return {
        liveConsumption: [...data].reverse(),
        liveConsumptionStats: getLiveConsumptionStats([...data]),
    };
}
