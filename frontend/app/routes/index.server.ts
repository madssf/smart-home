import {getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {Consumption, LiveConsumption, Price} from "~/routes/types";

export async function getCurrentPrice(): Promise<Price> {
    return await getRequest<Price>(apiRoutes.prices.current);
}

export async function getConsumption(): Promise<Consumption[]> {
    return await getRequest<Consumption[]>(apiRoutes.prices.consumption);
}

export async function getLiveConsumption(): Promise<LiveConsumption[]> {
    return await getRequest<LiveConsumption[]>(apiRoutes.prices.live_consumption);
}
