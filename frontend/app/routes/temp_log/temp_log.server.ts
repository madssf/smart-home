import {getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {TempLogType} from "~/routes/temp_log";

export async function getRoomTemperatureLogs(room_id: string): Promise<TempLogType[]> {
    return await getRequest<TempLogType[]>(apiRoutes.temperature_logs(room_id));

}
