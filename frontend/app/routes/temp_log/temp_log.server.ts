import {getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {TempLogType} from "~/routes/temp_log";

export async function getTemperatureLogs(): Promise<TempLogType[]> {
    return await getRequest<TempLogType[]>(apiRoutes.temperature_logs);
}
