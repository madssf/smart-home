import type {ActionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import type {PriceLevel, Schedule, Weekday} from "~/routes/schedules/types";
import {PRICE_LEVELS, WEEKDAYS} from "~/routes/schedules/types";
import ScheduleForm from "~/routes/schedules/components/scheduleForm";
import {routes} from "~/routes";
import {validateDays, validatePriceLevel, validateTimeWindows} from "~/routes/schedules/utils/utils";
import type {FormErrors} from "~/utils/types";
import {useLoaderData} from "@remix-run/react";
import React, {useState} from "react";
import {
    Accordion,
    AccordionButton,
    AccordionIcon,
    AccordionItem,
    AccordionPanel,
    Box,
    Button,
    Checkbox,
    Heading,
} from "@chakra-ui/react";
import {piTriggerRefresh} from "~/utils/piHooks";
import {createSchedule, deleteSchedule, getSchedules, updateSchedule} from "~/routes/schedules/schedules.server";
import {validateNonEmptyList, validateTemp} from "~/utils/validation";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {CheckCircleIcon} from "@chakra-ui/icons";

interface ResponseData {
    schedules: Schedule[];
    rooms: Room[];
}
export type ScheduleFormErrors = FormErrors<Schedule>

export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

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

export const loader: LoaderFunction = async () => {

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
    const [showNew, setShowNew] = useState( loaderData.schedules.length === 0);
    const [priceLevelFilters, setPriceLevelFilters] = useState<PriceLevel[]>([]);
    const [roomFilters, setRoomFilters] = useState<string[]>([]);
    const [dayFilters, setDayFilters] = useState<Weekday[]>([]);
    const activeFilters = priceLevelFilters.length > 0 || roomFilters.length > 0 || dayFilters.length > 0;

    const generateFilters = (): ((schedule: Schedule) => boolean)[] => {
        const filters = [];

        if (priceLevelFilters.length > 0) {
            filters.push((schedule: Schedule) => priceLevelFilters.includes(schedule.price_level));
        }

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
            <Accordion pb={4} allowToggle>
                <AccordionItem>
                    <h2>
                        <AccordionButton>
                            <Box flex='1' textAlign='left'>
                                Filters
                            </Box>
                            {activeFilters &&
                                <CheckCircleIcon />
                            }
                            <AccordionIcon />
                        </AccordionButton>
                    </h2>
                    <AccordionPanel pb={4}>
                        <div className="flex flex-col">
                            <label className="font-bold">Price levels</label>
                            <div className="flex">
                                {PRICE_LEVELS.map((level) => {
                                    return <Checkbox
                                        key={level}
                                        size="sm"
                                        className="mr-1"
                                        checked={priceLevelFilters.includes(level)}
                                        onChange={() => {
                                            if (priceLevelFilters.includes(level)) {
                                                setPriceLevelFilters((prev) => prev.filter((l) => l !== level));
                                            } else {
                                                setPriceLevelFilters((prev) => prev.concat([level]));
                                            }
                                        }}
                                    >
                                        {capitalizeAndRemoveUnderscore(level)}
                                    </Checkbox>;
                                })}
                            </div>
                        </div>
                        <div className="flex flex-col">
                            <label className="font-bold">Weekdays</label>
                            <div className="flex">
                                {WEEKDAYS.map((day) => {
                                    return <Checkbox
                                        key={day}
                                        size="sm"
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
                            <label className="font-bold">Rooms</label>
                            <div className="flex">
                                {loaderData.rooms.map((room) => {
                                    return <Checkbox
                                        key={room.id}
                                        size="sm"
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
                    </AccordionPanel>
                </AccordionItem>
            </Accordion>
        );
    };

    return (
        <div>
            <Heading className="pb-4">Schedules</Heading>
            {renderFilters()}
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
