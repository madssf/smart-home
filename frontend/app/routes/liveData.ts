import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {getLiveConsumption} from "~/routes/index.server";
import type {LiveConsumption} from "~/routes/types";

export interface LiveConsumptionData {
    liveConsumption: LiveConsumption[]
}

export const loader: LoaderFunction = async () => {
    const liveConsumption = await getLiveConsumption();

    return json<LiveConsumptionData>({
        liveConsumption,
    });
};
