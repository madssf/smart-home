import React from 'react';
import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Link, Outlet, useLoaderData, useParams} from "@remix-run/react";
import {Heading} from "@chakra-ui/react";
import {routes} from "~/routes";
import type {Room} from './rooms/types';
import {getRooms} from "~/routes/rooms/rooms.server";

type ResponseData = {
    rooms: Room[]
}

export const loader: LoaderFunction = async () => {

    return json<ResponseData>({
        rooms: await getRooms(),
    });

};

export const handle = {hydrate: true};

const TempLog = () => {
    const loaderData = useLoaderData<ResponseData>();
    const {room_id} = useParams();

    return (
        <div>
            <Heading className="pb-4">Temperature log</Heading>
            <div className="flex flex-row justify-center">
                {loaderData.rooms.map((room) => {
                    return <Link
                        key={room.id}
                        className={`mx-2 ${room_id === room.id ? 'font-bold cursor-auto': ''}`}
                        to={routes.TEMP_LOG.ROOM_ID(room.id)}
                    >
                        {room.name}
                    </Link>;
                })}
            </div>
          <Outlet />
        </div>
    );
};

export default TempLog;
