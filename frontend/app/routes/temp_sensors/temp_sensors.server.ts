import {createRequest, deleteRequest, getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {TempSensor} from "~/routes/temp_sensors/types";

export async function getTempSensors(): Promise<TempSensor[]> {
    return await getRequest<TempSensor[]>(apiRoutes.temp_sensors);
}

export async function createTempSensor(sensor: TempSensor): Promise<void> {
    return await createRequest<TempSensor>(apiRoutes.temp_sensors, sensor);
}

export async function deleteTempSensor(id: string): Promise<void> {
    return await deleteRequest(apiRoutes.temp_sensors, id);
}

