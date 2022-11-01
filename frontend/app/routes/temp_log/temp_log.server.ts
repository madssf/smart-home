import {getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {TempLogType, TimePeriod} from "~/routes/temp_log/$room_id";

export async function getRoomTemperatureLogs(room_id: string, time_period: TimePeriod): Promise<TempLogType[]> {
    return await getRequest<TempLogType[]>(apiRoutes.temperature_logs.room_id(room_id, time_period));

}
