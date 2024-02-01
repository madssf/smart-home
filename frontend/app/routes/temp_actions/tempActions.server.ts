import type {CreateRequest} from "~/fetcher/fetcher.server";
import {createRequest, deleteRequest, getRequest, updateRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {TempAction} from "~/routes/temp_actions/types";

export async function getTempActions(): Promise<TempAction[]> {
    return await getRequest<TempAction[]>(apiRoutes.temp_actions);
}

export async function createTempAction(action: CreateRequest<TempAction>): Promise<void> {
    return await createRequest<CreateRequest<TempAction>>(apiRoutes.temp_actions, action);
}

export async function updateTempAction(action: TempAction): Promise<void> {
    return await updateRequest<TempAction>(apiRoutes.temp_actions, action);
}

export async function deleteTempAction(id: string): Promise<void> {
    return await deleteRequest(apiRoutes.temp_actions, id);
}
