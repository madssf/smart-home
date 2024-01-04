import type {ActionFunctionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import type {Schedule, Weekday} from "~/routes/schedules/types";
import {WEEKDAYS} from "~/routes/schedules/types";
import ScheduleForm from "~/routes/schedules/components/scheduleForm";
import {routes} from "~/routes";
import {validateDays, validateTemps, validateTimeWindows} from "~/routes/schedules/utils/utils";
import type {FormErrors} from "~/utils/types";
import {Link, useLoaderData} from "@remix-run/react";
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import React, {useState} from "react";
import {piTriggerRefresh} from "~/utils/piHooks";
import {createSchedule, deleteSchedule, getSchedules, updateSchedule} from "~/routes/schedules/schedules.server";
import {validateNonEmptyList} from "~/utils/validation";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {PriceLevel} from "~/routes/types";
import {CheckCircleIcon} from "lucide-react";
import {Accordion, AccordionContent, AccordionItem, AccordionTrigger} from "~/components/ui/accordion";
import {Checkbox} from "~/components/ui/checkbox";
import {Button} from "~/components/ui/button";

interface ResponseData {
    schedules: Schedule[];
    rooms: Room[];
}
export type ScheduleFormErrors = FormErrors<Schedule>

export const handle = {hydrate: true};

export async function action({request}: ActionFunctionArgs) {

    const body = await request.formData();
    const id = body.get("id")?.toString();
    const days = body.getAll("days").map((day) => day.toString());
    const from = body.getAll("from").map((naiveTime) => naiveTime.toString());
    const to = body.getAll("to").map((naiveTime) => naiveTime.toString());
    const room_ids = body.getAll("room_ids").map((room_id) => room_id.toString());
    const temps = Object.values(PriceLevel).map((priceLevel) => {
        return {
            priceLevel: priceLevel as PriceLevel,
            temp: body.get(`temp_${priceLevel}`)?.toString(),
        };
    });

    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deleteSchedule(id!);
        await piTriggerRefresh();
        return redirect(routes.SCHEDULES.ROOT);
    }

    const validated = {
        temps: validateTemps(temps),
        days: validateDays(days),
        time_windows: validateTimeWindows(from, to),
        room_ids: validateNonEmptyList(room_ids),
    };

    if (!validated.temps.valid || !validated.days.valid || !validated.time_windows.valid || !validated.room_ids.valid) {
        return json<ScheduleFormErrors>(
            {
                id,
                days: !validated.days.valid ? validated.days.error : undefined,
                time_windows: !validated.time_windows.valid ? validated.time_windows.error : undefined,
                temps: !validated.temps.valid ? validated.temps.error : undefined,
                room_ids: !validated.room_ids.valid ? validated.room_ids.error : undefined,
            },
        );
    }

    const document: Omit<Schedule, 'id'> = {
        days: validated.days.data,
        time_windows: validated.time_windows.data,
        room_ids: validated.room_ids.data,
        temps: validated.temps.data,
    };

    if (!id) {
        await createSchedule(document);
    } else {
        await updateSchedule({id, ...document});
    }

    await piTriggerRefresh();

    return redirect(routes.SCHEDULES.ROOT);
}

export const loader: LoaderFunction = async () => {

    const schedules = await getSchedules();
    const sorted = schedules
        .sort((a, b) => {
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
    const [showNew, setShowNew] = useState( loaderData.schedules.length === 0);
    const [roomFilters, setRoomFilters] = useState<string[]>([]);
    const [dayFilters, setDayFilters] = useState<Weekday[]>([]);
    const activeFilters = roomFilters.length > 0 || dayFilters.length > 0;

    const generateFilters = (): ((schedule: Schedule) => boolean)[] => {
        const filters = [];

        if (roomFilters.length > 0) {
            filters.push((schedule: Schedule) => schedule.room_ids.some(id => roomFilters.includes(id)));
        }

        if (dayFilters.length > 0) {
            filters.push((schedule: Schedule) => schedule.days.some(day => dayFilters.includes(day.toUpperCase() as Weekday)));
        }

        return filters;
    };
    const filters = generateFilters();

    const renderSchedules = (schedules: Schedule[]) => {
        return schedules
            .filter((schedule) => {
                if (!activeFilters) {
                    return true;
                } else {
                   return filters.some((filter) => filter(schedule));
                }
            })
            .map((schedule) => {
            return (
                <ScheduleForm key={schedule.id} schedule={schedule} rooms={loaderData.rooms}/>
            );
        });
    };

    const renderFilters = () => {
        return (
            <Accordion type="single" className="pb-4">
                <AccordionItem value="filters">
                    <h2>
                        <AccordionTrigger>
                            <div className="flex-1 text-left">
                                Filters
                            </div>
                            {activeFilters &&
                                <CheckCircleIcon />
                            }
                        </AccordionTrigger>
                    </h2>
                    <AccordionContent className="pb-4">
                        <div className="flex flex-col">
                            <p className="font-bold">Weekdays</p>
                            <div className="flex">
                                {WEEKDAYS.map((day) => {
                                    return <Checkbox
                                        key={day}
                                        className="mr-1"
                                        checked={dayFilters.includes(day)}
                                        onChange={() => {
                                            if (dayFilters.includes(day)) {
                                                setDayFilters((prev) => prev.filter((d) => d !== day));
                                            } else {
                                                setDayFilters((prev) => prev.concat([day]));
                                            }
                                        }}
                                    >
                                        {capitalizeAndRemoveUnderscore(day)}
                                    </Checkbox>;
                                })}
                            </div>
                        </div>
                        <div className="flex flex-col">
                            <p className="font-bold">Rooms</p>
                            <div className="flex">
                                {loaderData.rooms.map((room) => {
                                    return <Checkbox
                                        key={room.id}
                                        className="mr-1"
                                        checked={roomFilters.includes(room.id)}
                                        onChange={() => {
                                            if (roomFilters.includes(room.id)) {
                                                setRoomFilters((prev) => prev.filter((r) => r !== room.id));
                                            } else {
                                                setRoomFilters((prev) => prev.concat([room.id]));
                                            }
                                        }}
                                    >
                                        {capitalizeAndRemoveUnderscore(room.name)}
                                    </Checkbox>;
                                })}
                            </div>
                        </div>
                    </AccordionContent>
                </AccordionItem>
            </Accordion>
        );
    };

    return (
        <div>
            <h1 className="pb-4">Schedules</h1>
            {
                loaderData.rooms.length === 0 ?
                    <p>No rooms yet, please <Link to={routes.ROOMS.ROOT}>add one</Link> before adding a schedule</p>
                    :
                    <>
                        {renderFilters()}
                        {renderSchedules(loaderData.schedules as Schedule[])}
                        <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add schedule'}</Button>
                        {
                            showNew &&
                            <ScheduleForm rooms={loaderData.rooms} />
                        }
                    </>
            }

        </div>
    );
};

export default Schedules;
