import React, {useState} from 'react';
import {routes} from "~/routes";
import type {Plug} from "~/routes/plugs/types";
import type {FormErrors} from "~/utils/types";
import type {ActionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import PlugForm from "~/routes/plugs/components/plugForm";
import {useLoaderData} from "@remix-run/react";
import {Button, Heading} from "@chakra-ui/react";
import {validateIpAddress, validateNonEmptyString} from "~/utils/validation";
import {piTriggerRefresh} from "~/utils/piHooks";
import {createPlug, deletePlug, getPlugs, updatePlug} from "~/routes/plugs/plugs.server";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";

interface ResponseData {
    plugs: Plug[];
    rooms: Room[];
}

export type PlugFormErrors = FormErrors<Plug>;

export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const name = body.get("name")?.toString();
    const ip = body.get("ip")?.toString();
    const username = body.get("username")?.toString();
    const password = body.get("password")?.toString();
    const roomId = body.get("room_id")?.toString();


    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deletePlug(id!);
        await piTriggerRefresh();
        return redirect(routes.PLUGS.ROOT);
    }

    const validated = {
        name: validateNonEmptyString(name),
        ip: validateIpAddress(ip),
        username: validateNonEmptyString(username),
        password: validateNonEmptyString(password),
        roomId: validateNonEmptyString(roomId),
    };

    if (!validated.name.valid || !validated.ip.valid || !validated.username.valid || !validated.password.valid || !validated.roomId.valid) {
        return json<PlugFormErrors>(
            {
                id,
                name: !validated.name.valid ? validated.name.error : undefined,
                ip: !validated.ip.valid ? validated.ip.error : undefined,
                username: !validated.username.valid ? validated.username.error : undefined,
                password: !validated.password.valid ? validated.password.error : undefined,
                room_id: !validated.roomId.valid ? validated.roomId.error : undefined,
            },
        );
    }

    const document: Omit<Plug, 'id'> = {
        name: validated.name.data,
        ip: validated.ip.data,
        username: validated.username.data,
        password: validated.password.data,
        room_id: validated.roomId.data,
    };

    if (!id) {
        await createPlug(document);
    } else {
        await updatePlug({...document, id});
    }
    await piTriggerRefresh();
    return redirect(routes.PLUGS.ROOT);
}

export const loader: LoaderFunction = async () => {

    const plugs = await getPlugs();
    const rooms = await getRooms();

    return json<ResponseData>({
        plugs,
        rooms,
    });

};

const Plugs = () => {

    const loaderData = useLoaderData<ResponseData>();
    const [showNew, setShowNew] = useState(false);

    const renderPlugs = (plugs: Plug[]) => {
        return plugs.map((plug) => {
            return (
                <PlugForm key={plug.id} plug={plug} rooms={loaderData.rooms}/>
            );
        });
    };

    return (
        <div>
            <Heading className="pb-4">Plugs</Heading>
            {renderPlugs(loaderData.plugs)}
            <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add plug'}</Button>
            {
                showNew &&
                <PlugForm rooms={loaderData.rooms} />
            }
        </div>
    );
};

export default Plugs;
