import type {CreateRequest} from "~/fetcher/fetcher.server";
import {createRequest, deleteRequest, getRequest, updateRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {ButtonType} from "~/routes/buttons/types";


export async function getButtons(): Promise<ButtonType[]> {
    return await getRequest<ButtonType[]>(apiRoutes.buttons);
}

export async function createButton(button: CreateRequest<ButtonType>): Promise<void> {
    return await createRequest<CreateRequest<ButtonType>>(apiRoutes.buttons, button);
}

export async function updateButton(button: ButtonType): Promise<void> {
    return await updateRequest<ButtonType>(apiRoutes.buttons, button);
}

export async function deleteButton(id: string): Promise<void> {
    return await deleteRequest(apiRoutes.buttons, id);
}
