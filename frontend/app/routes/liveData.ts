import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {getLiveConsumption} from "~/routes/index.server";
import type {LiveConsumption} from "~/routes/types";

export interface LiveConsumptionData {
    liveConsumption: LiveConsumption[]
    liveConsumptionStats: LiveConsumptionStats
}

export type LiveConsumptionChange = 'UP' | 'NONE' | 'DOWN'
export type LiveConsumptionStats = { consumption: number | null, consumptionChange: LiveConsumptionChange, consumptionTime: string | null }

export const loader: LoaderFunction = async () => {
    const liveConsumption = await getLiveConsumption();

    return json<LiveConsumptionData>({
        liveConsumption: [...liveConsumption].reverse(),
        liveConsumptionStats: getLiveConsumptionStats([...liveConsumption]),
    });
};

const getLiveConsumptionStats = (data: LiveConsumption[]): LiveConsumptionStats => {
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
