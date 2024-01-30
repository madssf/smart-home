import type {Room} from "~/routes/rooms/types";
import type {TempSensor} from "~/routes/temp_sensors/types";
import type {ActionFunction, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import {getRooms} from "~/routes/rooms/rooms.server";
import {piTriggerRefresh} from "~/utils/piHooks";
import {routes} from "~/routes";
import {validateNonEmptyString} from "~/utils/validation";
import {createTempSensor, deleteTempSensor, getTempSensors} from "~/routes/temp_sensors/temp_sensors.server";
import type {FormErrors} from "~/utils/types";
import {useLoaderData} from "@remix-run/react";
import React, {useState} from "react";
import TempSensorForm from "~/routes/temp_sensors/components/tempSensorForm";
import {Button} from "~/components/ui/button";

interface ResponseData {
    rooms: Room[],
    temp_sensors: TempSensor[],
}

export const handle = {hydrate: true};

export const action: ActionFunction = async ({request}) => {

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const room_id = body.get("room_id")?.toString();
    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deleteTempSensor(id!);
        await piTriggerRefresh();
        return redirect(routes.TEMP_SENSORS.ROOT);
    }

    const validated = {
        id: validateNonEmptyString(id),
        room_id: validateNonEmptyString(room_id),
    };

    if (!validated.id.valid || !validated.room_id.valid) {
        return json<FormErrors<TempSensor>>(
            {
                id: !validated.id.valid ? validated.id.error : undefined,
                room_id: !validated.room_id.valid ? validated.room_id.error : undefined,
            },
        );
    }

    const document: TempSensor = {
        id: validated.id.data,
        room_id: validated.room_id.data,
        battery_level: null,
    };

    await createTempSensor(document);
    await piTriggerRefresh();
    return redirect(routes.TEMP_SENSORS.ROOT);

};

export const loader: LoaderFunction = async () => {

    const rooms = await getRooms();
    const temp_sensors = await getTempSensors();

    return json<ResponseData>({rooms, temp_sensors});
};


const TempSensors = () => {

    const loaderData = useLoaderData<ResponseData>();
    const [showNew, setShowNew] = useState(false);


    const renderTempSensors = (tempSensors: TempSensor[], rooms: Room[]) => {
        return tempSensors.map((sensor) => {
            return (
                <TempSensorForm key={sensor.id} sensor={sensor} rooms={rooms}/>
            );
        });
    };

    return (
        <div>
            <h1 className="pb-4">Sensors</h1>
            {renderTempSensors(loaderData.temp_sensors, loaderData.rooms)}
            <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add sensor'}</Button>
            {
                showNew &&
                <TempSensorForm rooms={loaderData.rooms} />
            }
        </div>
    );
};

export default TempSensors;
