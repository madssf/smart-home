import {useEffect, useRef, useState} from 'react';
import type {Schedule, TimeWindow} from "~/routes/schedules/types";
import {getWeekdayName, WEEKDAYS} from "~/routes/schedules/types";
import TimeForm from "~/routes/schedules/components/timeForm";
import {routes} from "~/routes";
import type {ScheduleFormErrors} from "~/routes/schedules";
import {Form, useActionData} from "@remix-run/react";
import {capitalizeAndRemoveUnderscore, formatPriceLevel} from '~/utils/formattingUtils';
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {Room} from "~/routes/rooms/types";
import {PriceLevel, sortedPriceLevels} from '~/routes/types';
import {DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuTrigger} from "~/components/ui/dropdown-menu";
import {Checkbox} from "~/components/ui/checkbox";
import {Button} from "~/components/ui/button";
import {Input} from "~/components/ui/input";
import {useNavigation} from "react-router";
import {Label} from '~/components/ui/label';
import {Trash} from "lucide-react";

export interface ScheduleFormProps {
    schedule?: Schedule
    rooms: Room[]
}

const ScheduleForm = ({schedule, rooms}: ScheduleFormProps) => {
    const actionData = useActionData<ScheduleFormErrors>();
    const navigation = useNavigation();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(schedule);

    const getInitialPriceLevels = (schedule: Schedule | undefined): PriceLevel[] => {
        if (schedule === undefined) {
            return [PriceLevel.VeryCheap, PriceLevel.VeryExpensive];
        } else {
            return Object.keys(schedule.temps) as PriceLevel[];
        }
    };

    const [activePriceLevels, setActivePriceLevels] = useState<PriceLevel[]>(getInitialPriceLevels(schedule));



    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<ScheduleFormErrors | null>(null);
    const defaultTimeWindow: TimeWindow = ["16:00:00", "20:00:00"];

    useEffect(() => {
        if (actionData && !schedule && !actionData.id) {
            setErrors(actionData);
        } else if (actionData && schedule?.id === actionData.id) {
            setErrors(actionData);
        } else {
            setErrors(null);
        }
    }, [schedule, actionData]);

    useEffect(() => {
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
        setHoursList(schedule?.time_windows ?? [defaultTimeWindow]);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [navigation]);

    const [hoursList, setHoursList] = useState(schedule?.time_windows ?? []);

    const handleRemoveTimeWindow = (toRemove: TimeWindow) => {
        setHoursList((prev) => prev.filter((existing) => {
            return existing[0] !== toRemove[0] && existing[1] !== toRemove[1];
        }));
    };

    const addTimeWindow = () => {
        setHoursList((prev) => prev.concat([defaultTimeWindow]));
    };

    const AddPriceLevelMenu = () => {
        return <DropdownMenu>
            <DropdownMenuTrigger
                aria-label='Add price level'
                asChild
            >
                <Button>Add price level</Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
                {
                    Object.keys(PriceLevel)
                        .filter(priceLevel => !activePriceLevels.includes(priceLevel as PriceLevel))
                        .map((priceLevel) => {
                            return (
                                <DropdownMenuLabel
                                    key={priceLevel}
                                    onClick={() =>
                                        setActivePriceLevels(
                                            (prev) => prev.concat([priceLevel as PriceLevel]),
                                        )
                                    }
                                >
                                    {formatPriceLevel(priceLevel as PriceLevel)}
                                </DropdownMenuLabel>
                            );
                        })
                }
            </DropdownMenuContent>
        </DropdownMenu>;
    };

    return (
        <Form className="mb-2 flex flex-col gap-4" ref={formRef} method="post" action={routes.SCHEDULES.ROOT} reloadDocument>
            <input hidden readOnly name="id" value={schedule?.id}/>
            <div className="flex flex-col gap-2">
                <p className="font-bold" >Rooms</p>
                <div className="flex flex-col space-y-3">
                    {rooms.map((room) => {
                        return <div
                            key={schedule?.id + room.id}
                            className="flex flex-row space-x-1"
                        >
                            <Checkbox
                                key={schedule?.id + room.id}
                                className="mr-1"
                                id={room.id}
                                name="room_ids"
                                value={room.id}
                                defaultChecked={schedule?.room_ids.includes(room.id)}
                            />
                            <Label htmlFor={room.id}>{capitalizeAndRemoveUnderscore(room.name)}</Label>
                        </div>;
                    })}
                </div>
                {
                    !!errors?.room_ids &&
                    <p color="tomato">{errors.room_ids}</p>
                }
            </div>
            <div className="flex flex-col gap-2">
                <p className="font-bold">Weekdays</p>
                <div className="flex flex-col space-y-3">
                {WEEKDAYS.map((day) => {
                    return <div
                        key={day}
                        className="flex flex-row space-x-1"
                    >
                            <Checkbox
                                className="mr-1"
                                id={day}
                                name="days"
                                value={day}
                                defaultChecked={schedule?.days.map(str => str.toUpperCase()).includes(day.toUpperCase())}
                            />
                        <Label htmlFor={day}>{getWeekdayName(day)}</Label>
                    </div>;
                })}
                </div>
                {
                    !!errors?.days &&
                    <p color="tomato">{errors.days}</p>
                }
            </div>
            <div>
                <p className="font-bold">Time windows</p>
            {
                <div className="ml-2 mb-1">
                    {
                        (hoursList).map((window, i) => {
                            return <TimeForm
                                key={i}
                                window={window}
                                handleRemove={() => handleRemoveTimeWindow(window)}
                                handleAdd={i === hoursList.length - 1 ? () => addTimeWindow() : undefined}
                            />;
                        })
                    }
                    {
                        hoursList.length === 0 &&
                        <Button variant="outline" type="button" onClick={addTimeWindow}>Add time window</Button>
                    }
                </div>
            }
                {
                    !!errors?.time_windows &&
                    <p color="tomato">{errors.time_windows}</p>
                }
            </div>
            <div
                className="mb-4"
            >
                <p className="font-bold">Temperature by price level</p>
                <div className="flex flex-col gap-2 ml-2 mb-1">
                {
                    sortedPriceLevels(activePriceLevels).map((price_level) => {
                        return (
                            <div key={price_level} className="grid grid-cols-[100px_100px_40px_30px] items-center">
                                <p className="text-sm">{formatPriceLevel(price_level)}</p>
                                <div
                                    className="flex items-center"
                                >
                                    <Input
                                        style={{width: "70px"}}
                                        type="number"
                                        min="1"
                                        max="100"
                                        step="1"
                                        name={`temp_${price_level}`}
                                        defaultValue={schedule?.temps[price_level]}
                                    />
                                    <span
                                        className="ml-1 text-gray-600 dark:text-gray-400"
                                    >
                                        °C
                                    </span>
                                </div>
                                <Button
                                    className="ml-2"
                                    size="icon"
                                    variant="outline"
                                    type="button"
                                    onClick={() => setActivePriceLevels((prev) => prev.filter(p => p !== price_level))}
                                >
                                    <Trash
                                        className="w-4 h-4"
                                    />
                                </Button>
                            </div>
                    );
                    })
                }
                {
                    activePriceLevels.length < Object.keys(PriceLevel).length &&
                        <div>
                            <AddPriceLevelMenu />
                        </div>
                }
                </div>
                {
                    !!errors?.temps &&
                    <p color="tomato">{errors.temps}</p>
                }
            </div>
            {
                !!errors?.other &&
                <p color="tomato">{errors.other}</p>
            }

            <div>
                <Button className="mr-1" type="submit" name="intent" value={isNew ? 'create' : 'update'}
                        disabled={isCreating || isUpdating}>{isNew ? "Add" : "Update"}</Button>
                {
                    !isNew &&
                    <Button variant="outline" type="submit" name="intent" value="delete"
                            disabled={isDeleting}>{isDeleting ? 'Deleting...' : 'Delete'}</Button>

                }
            </div>
        </Form>
    );
};

export default ScheduleForm;
