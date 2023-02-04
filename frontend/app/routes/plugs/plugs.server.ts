import type {Plug} from "~/routes/plugs/types";
import type {CreateRequest} from "~/fetcher/fetcher.server";
import {createRequest, deleteRequest, getRequest, updateRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";


export async function getPlugs(): Promise<Plug[]> {
    return await getRequest<Plug[]>(apiRoutes.plugs);
}

export async function createPlug(plug: CreateRequest<Plug>): Promise<void> {
    return await createRequest<CreateRequest<Plug>>(apiRoutes.plugs, plug);
}

export async function updatePlug(plug: Plug): Promise<void> {
    return await updateRequest<Plug>(apiRoutes.plugs, plug);
}

export async function deletePlug(id: string): Promise<void> {
    return await deleteRequest(apiRoutes.plugs, id);
}
