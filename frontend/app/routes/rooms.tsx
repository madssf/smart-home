import React, {useState} from 'react';
import type {ActionFunction, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {createRoom, deleteRoom, getRooms, updateRoom} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";
import type {FormErrors} from "~/utils/types";
import {piTriggerRefresh} from "~/utils/piHooks";
import {routes} from "~/routes";
import {validateNonEmptyString} from "~/utils/validation";
import type {PlugFormErrors} from "~/routes/plugs";
import {useLoaderData} from "@remix-run/react";
import RoomForm from "~/routes/rooms/components/roomForm";
import {Button, Heading} from "@chakra-ui/react";

interface ResponseData {
    rooms: Room[];
}

export type RoomFormErrors = FormErrors<Room>

export const handle = {hydrate: true};


export const action: ActionFunction = async ({request}) => {
    await requireUserId(request);

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const name = body.get("name")?.toString();
    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deleteRoom(id!);
        await piTriggerRefresh();
        return redirect(routes.ROOMS.ROOT);
    }

    const validated = {
        name: validateNonEmptyString(name),
    };

    if (!validated.name.valid) {
        return json<PlugFormErrors>(
            {
                id,
                name: !validated.name.valid ? validated.name.error : undefined,
            },
        );
    }

    const document: Omit<Room, 'id'> = {
        name: validated.name.data,
    };

    if (!id) {
        await createRoom(document);
    } else {
        await updateRoom({...document, id});
    }
    await piTriggerRefresh();
    return redirect(routes.ROOMS.ROOT);

};

export const loader: LoaderFunction = async ({request}) => {
    await requireUserId(request);

    const rooms = await getRooms();
    
    return json<ResponseData>({rooms});
};

const Rooms = () => {

    const loaderData = useLoaderData<ResponseData>();
    const [showNew, setShowNew] = useState(false);


    const renderRooms = (rooms: Room[]) => {
        return rooms.map((room) => {
            return (
                <RoomForm key={room.id} room={room}/>
            );
        });
    };

    return (
        <div>
            <Heading className="pb-4">Rooms</Heading>
            {renderRooms(loaderData.rooms)}
            <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add room'}</Button>
            {
                showNew &&
                <RoomForm />
            }
        </div>
    );
};

export default Rooms;
