import type {DataOrError} from "~/fetcher/fetcher.server";
import {getRequest, getRequestOrError} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {ActiveSchedule, Consumption, LiveConsumption, PlugStatus, PriceInfo, RoomTemp} from "~/routes/types";

export async function getCurrentPriceOrError(): Promise<DataOrError<PriceInfo>> {
    return await getRequestOrError<PriceInfo>(apiRoutes.prices.current);
}

export async function getConsumptionOrError(): Promise<DataOrError<Consumption[]>> {
    return await getRequestOrError<Consumption[]>(apiRoutes.prices.consumption);
}

export async function getLiveConsumption(): Promise<LiveConsumption[]> {
    return await getRequest<LiveConsumption[]>(apiRoutes.prices.live_consumption);
}

export async function getRoomTemps(): Promise<RoomTemp[]> {
    return await getRequest<RoomTemp[]>(apiRoutes.temperature_logs.current);
}

export async function getPlugStatuses(): Promise<PlugStatus[]> {
    return await getRequest<PlugStatus[]>(apiRoutes.plug_status);
}

export async function getActiveSchedules(): Promise<ActiveSchedule[]> {
    return await getRequest<ActiveSchedule[]>(apiRoutes.active_schedules);
}
