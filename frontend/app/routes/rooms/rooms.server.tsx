import type {CreateRequest} from "~/fetcher/fetcher.server";
import {createRequest, deleteRequest, getRequest, updateRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {Room} from "~/routes/rooms/types";

export async function getRooms(): Promise<Room[]> {
    return await getRequest<Room[]>(apiRoutes.rooms);
}

export async function createRoom(room: CreateRequest<Room>): Promise<void> {
    return await createRequest<CreateRequest<Room>>(apiRoutes.rooms, room);
}

export async function updateRoom(room: Room): Promise<void> {
    return await updateRequest<Room>(apiRoutes.rooms, room);
}

export async function deleteRoom(id: string): Promise<void> {
    return await deleteRequest(apiRoutes.rooms, id);
}
