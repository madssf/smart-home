import type {ActionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import type {Schedule} from "~/routes/schedules/types";
import {PRICE_LEVELS} from "~/routes/schedules/types";
import {requireUserId} from "~/utils/sessions.server";
import ScheduleForm from "~/routes/schedules/components/scheduleForm";
import {routes} from "~/routes";
import {validateDays, validatePriceLevel, validateTimeWindows} from "~/routes/schedules/utils/utils";
import type {FormErrors} from "~/utils/types";
import {useLoaderData} from "@remix-run/react";
import React, {useState} from "react";
import {Button, Heading} from "@chakra-ui/react";
import {piTriggerRefresh} from "~/utils/piHooks";
import {createSchedule, deleteSchedule, getSchedules, updateSchedule} from "~/routes/schedules/schedules.server";
import {validateNonEmptyList, validateTemp} from "~/utils/validation";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";

interface ResponseData {
    schedules: Schedule[];
    rooms: Room[];
}
export type ScheduleFormErrors = FormErrors<Schedule>

export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

    await requireUserId(request);

    const body = await request.formData();
    const id = body.get("id")?.toString();
    const priceLevel = body.get("price_level")?.toString();
    const days = body.getAll("days").map((day) => day.toString());
    const from = body.getAll("from").map((naiveTime) => naiveTime.toString());
    const to = body.getAll("to").map((naiveTime) => naiveTime.toString());
    const room_ids = body.getAll("room_ids").map((room_id) => room_id.toString());
    const temp = body.get("temp")?.toString();

    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deleteSchedule(id!);
        await piTriggerRefresh();
        return redirect(routes.SCHEDULES.ROOT);
    }

    const validated = {
        priceLevel: validatePriceLevel(priceLevel),
        days: validateDays(days),
        time_windows: validateTimeWindows(from, to),
        room_ids: validateNonEmptyList(room_ids),
        temp: validateTemp(temp),
    };

    if (!validated.temp.valid || !validated.days.valid || !validated.time_windows.valid || !validated.priceLevel.valid || !validated.room_ids.valid) {
        return json<ScheduleFormErrors>(
            {
                id,
                days: !validated.days.valid ? validated.days.error : undefined,
                time_windows: !validated.time_windows.valid ? validated.time_windows.error : undefined,
                price_level: !validated.priceLevel.valid ? validated.priceLevel.error : undefined,
                room_ids: !validated.room_ids.valid ? validated.room_ids.error : undefined,
                temp: !validated.temp.valid ? validated.temp.error : undefined,
            },
        );
    }

    const document: Omit<Schedule, 'id'> = {
        days: validated.days.data,
        time_windows: validated.time_windows.data,
        price_level: validated.priceLevel.data,
        room_ids: validated.room_ids.data,
        temp: validated.temp.data,
    };

    if (!id) {
        await createSchedule(document);
    } else {
        await updateSchedule({id, ...document});
    }

    await piTriggerRefresh();

    return redirect(routes.SCHEDULES.ROOT);
}

export const loader: LoaderFunction = async ({request}) => {

    await requireUserId(request);

    const schedules = await getSchedules();
    const sorted = schedules
        .sort((a, b) => {
        if (a.days.length === b.days.length) {
            return PRICE_LEVELS.indexOf(a.price_level) - PRICE_LEVELS.indexOf(b.price_level);
        }
        return b.days.length - a.days.length;
    });

    const rooms = await getRooms();

    return json<ResponseData>({
        schedules: sorted,
        rooms,
    });

};

const Schedules = () => {

    const loaderData = useLoaderData<ResponseData>();
    const [showNew, setShowNew] = useState(false);

    const renderSchedules = (schedules: Schedule[]) => {
        return schedules
            .map((schedule) => {
            return (
                <ScheduleForm key={schedule.id} schedule={schedule} rooms={loaderData.rooms}/>
            );
        });
    };

    return (
        <div>
            <Heading className="pb-4">Schedules</Heading>
            {renderSchedules(loaderData.schedules)}
            <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add schedule'}</Button>
            {
                showNew &&
                <ScheduleForm rooms={loaderData.rooms} />
            }
        </div>
    );
};

export default Schedules;
