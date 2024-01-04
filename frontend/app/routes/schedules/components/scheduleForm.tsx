import {useEffect, useRef, useState} from 'react';
import type {Schedule, TimeWindow} from "~/routes/schedules/types";
import {WEEKDAYS} from "~/routes/schedules/types";
import TimeForm from "~/routes/schedules/components/timeForm";
import {routes} from "~/routes";
import type {ScheduleFormErrors} from "~/routes/schedules";
import {Form, useActionData} from "@remix-run/react";
import {capitalizeAndRemoveUnderscore, formatPriceLevel} from '~/utils/formattingUtils';
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {Room} from "~/routes/rooms/types";
import {PriceLevel} from '~/routes/types';
import {DropdownMenu, DropdownMenuContent, DropdownMenuLabel, DropdownMenuTrigger} from "~/components/ui/dropdown-menu";
import {Checkbox} from "~/components/ui/checkbox";
import {Button} from "~/components/ui/button";
import {Input} from "~/components/ui/input";
import {useNavigation} from "react-router";

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

    const renderPriceLevelMenu = () => {
        return <DropdownMenu>
            <DropdownMenuTrigger
                aria-label='Add price level'
            >
                Add price level
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
        <Form className="mb-2" ref={formRef} method="post" action={routes.SCHEDULES.ROOT}>
            <input hidden readOnly name="id" value={schedule?.id}/>
            <div className="flex flex-col">
                <p className="font-bold" >Rooms</p>
                <div className="flex">
                    {rooms.map((room) => {
                        return <Checkbox
                            key={schedule?.id + room.id}
                            className="mr-1"
                            id={room.id}
                            name="room_ids"
                            value={room.id}
                            defaultChecked={schedule?.room_ids.includes(room.id)}>
                            {room.name}
                        </Checkbox>;
                    })}
                </div>
                {
                    !!errors?.room_ids &&
                    <p color="tomato">{errors.room_ids}</p>
                }
            </div>
            <div className="flex flex-col">
                <p className="font-bold">Weekdays</p>
                <div className="flex">
                {WEEKDAYS.map((day) => {
                    return <Checkbox
                        key={schedule?.id + day}
                        className="mr-1"
                        id={day}
                        name="days"
                        value={day}
                        defaultChecked={schedule?.days.map(str => str.toUpperCase()).includes(day.toUpperCase())}>
                        {capitalizeAndRemoveUnderscore(day)}
                    </Checkbox>;
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
            <div>
                <p className="font-bold">Temperature by price level</p>
                <div className="ml-2 mb-1">
                {
                    activePriceLevels.map((price_level, i) => {
                        return (
                            <div key={price_level} className="grid grid-cols-[80px_120px_40px_30px] items-center">
                                <p className="text-sm">{formatPriceLevel(price_level)}</p>
                                <div>
                                    <Input
                                        style={{width: "70px"}}
                                        type="number"
                                        min="1"
                                        max="30"
                                        step="1"
                                        name={`temp_${price_level}`}
                                        defaultValue={schedule?.temps[price_level]}
                                    />
                                    <p>°C</p>
                                </div>
                                <Button
                                    size="sm"
                                    variant="outline"
                                    type="button"
                                    className="mx-1"
                                    onClick={() => setActivePriceLevels((prev) => prev.filter(p => p !== price_level))}
                                >
                                    ❌
                                </Button>
                                {
                                    i === activePriceLevels.length - 1 &&
                                        renderPriceLevelMenu()
                                }
                            </div>
                        );
                    })
                }
                {
                    activePriceLevels.length === 0 &&
                        <div>
                            {renderPriceLevelMenu()}
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
