import type {SimpleResult} from "~/fetcher/fetcher.server";
import {getRequest, getRequestOrError} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {
    ActiveSchedule,
    Consumption,
    EnrichedRoomData,
    LiveConsumption,
    PlugStatus,
    PriceInfo,
    RoomTemp,
} from "~/routes/types";
import type {Room} from "~/routes/rooms/types";
import {fromLiveConsumption, LiveConsumptionData} from "~/routes/liveData";

export async function getCurrentPriceOrError(): Promise<SimpleResult<PriceInfo>> {
    return await getRequestOrError<PriceInfo>(apiRoutes.prices.current);
}

export async function getConsumptionOrError(): Promise<SimpleResult<Consumption[]>> {
    return await getRequestOrError<Consumption[]>(apiRoutes.prices.consumption);
}

export async function getLiveConsumption(): Promise<LiveConsumptionData> {
    const result = await getRequest<LiveConsumption[]>(apiRoutes.prices.live_consumption);
    return fromLiveConsumption(result);
}

export async function getRoomTemps(): Promise<RoomTemp[]> {
    return await getRequest<RoomTemp[]>(apiRoutes.temperature_logs.current);
}

export async function getPlugStatuses(): Promise<SimpleResult<PlugStatus[]>> {
    return await getRequestOrError<PlugStatus[]>(apiRoutes.plug_status);
}

export async function getActiveSchedules(): Promise<ActiveSchedule[]> {
    return await getRequest<ActiveSchedule[]>(apiRoutes.active_schedules);
}

export const enrichRoomData = (
    room: Room,
    activeSchedules: ActiveSchedule[],
    roomTemps: RoomTemp[],
    plugStatusesResult: SimpleResult<PlugStatus[]>,
): EnrichedRoomData => {
    const plugStatuses = plugStatusesResult === 'ERROR' ? [] : plugStatusesResult;
    return {
        ...room,
        activeSchedule: activeSchedules.find(schedule => schedule.room_id === room.id) ?? null,
        temp: roomTemps.find(room_temp => room_temp.room_id === room.id) ?? null,
        plugStatuses: plugStatuses.filter((plug) => plug.room_id === room.id),

    };
};
