import type {CreateRequest} from "~/fetcher/fetcher.server";
import {createRequest, deleteRequest, getRequest, updateRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {Schedule} from "~/routes/schedules/types";

export async function getSchedules(): Promise<Schedule[]> {
    return await getRequest<Schedule[]>(apiRoutes.schedules);
}

export async function createSchedule(schedule: CreateRequest<Schedule>): Promise<void> {
    return await createRequest<CreateRequest<Schedule>>(apiRoutes.schedules, schedule);
}

export async function updateSchedule(schedule: Schedule): Promise<void> {
    return await updateRequest<Schedule>(apiRoutes.schedules, schedule);
}

export async function deleteSchedule(id: string): Promise<void> {
    return await deleteRequest(apiRoutes.schedules, id);
}
