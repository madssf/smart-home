import React, {useEffect, useRef, useState} from 'react';
import type {Schedule, TimeWindow} from "~/routes/schedules/types";
import {WEEKDAYS} from "~/routes/schedules/types";
import TimeForm from "~/routes/schedules/components/timeForm";
import {routes} from "~/routes";
import type {ScheduleFormErrors} from "~/routes/schedules";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {
    Button,
    Checkbox,
    IconButton,
    Input,
    InputGroup,
    InputRightAddon,
    Menu,
    MenuButton,
    MenuItem,
    MenuList,
    Text,
} from "@chakra-ui/react";
import {capitalizeAndRemoveUnderscore, formatPriceLevel} from '~/utils/formattingUtils';
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {Room} from "~/routes/rooms/types";
import {PriceLevel} from '~/routes/types';
import {SmallAddIcon, SmallCloseIcon} from "@chakra-ui/icons";

export interface ScheduleFormProps {
    schedule?: Schedule
    rooms: Room[]
}

const ScheduleForm = ({schedule, rooms}: ScheduleFormProps) => {
    const actionData = useActionData<ScheduleFormErrors>();
    const transition = useTransition();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(transition, schedule);

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
    }, [transition]);

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
        return <Menu>
            <MenuButton
                as={IconButton}
                aria-label='Add price level'
                icon={<SmallAddIcon />}
                size="sm"
            >
            </MenuButton>
            <MenuList>
                {
                    Object.keys(PriceLevel)
                        .filter(priceLevel => !activePriceLevels.includes(priceLevel as PriceLevel))
                        .map((priceLevel) => {
                            return (
                                <MenuItem
                                    key={priceLevel}
                                    as={Button}
                                    type='button'
                                    variant='outline'
                                    size='small'
                                    onClick={() =>
                                        setActivePriceLevels(
                                            (prev) => prev.concat([priceLevel as PriceLevel]),
                                        )
                                    }
                                >
                                    {formatPriceLevel(priceLevel as PriceLevel)}
                                </MenuItem>
                            );
                        })
                }
            </MenuList>
        </Menu>;
    };

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.SCHEDULES.ROOT}>
            <input hidden readOnly name="id" value={schedule?.id}/>
            <div className="flex flex-col">
                <label className="font-bold">Rooms</label>
                <div className="flex">
                    {rooms.map((room) => {
                        return <Checkbox
                            key={schedule?.id + room.id}
                            size="sm"
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
                    <Text color="tomato">{errors.room_ids}</Text>
                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold">Weekdays</label>
                <div className="flex">
                {WEEKDAYS.map((day) => {
                    return <Checkbox
                        key={schedule?.id + day}
                        size="sm"
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
                    <Text color="tomato">{errors.days}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Time windows</label>
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
                        <IconButton icon={<SmallAddIcon />} aria-label='Add time window' size="sm" type="button" onClick={addTimeWindow} />
                    }
                </div>
            }
                {
                    !!errors?.time_windows &&
                    <Text color="tomato">{errors.time_windows}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Temperature by price level</label>
                <div className="ml-2 mb-1">
                {
                    activePriceLevels.map((price_level, i) => {
                        return (
                            <div key={price_level} className="grid grid-cols-[80px_120px_40px_30px] items-center">
                                <Text fontSize="small">{formatPriceLevel(price_level)}</Text>
                                <InputGroup>
                                    <Input
                                        style={{width: "70px"}}
                                        type="number"
                                        min="1"
                                        max="30"
                                        step="1"
                                        name={`temp_${price_level}`}
                                        defaultValue={schedule?.temps[price_level]}
                                    />
                                    <InputRightAddon children="°C" />
                                </InputGroup>
                                <Button
                                    size="sm"
                                    variant="outline"
                                    type="button"
                                    className="mx-1"
                                    onClick={() => setActivePriceLevels((prev) => prev.filter(p => p !== price_level))}
                                >
                                    <SmallCloseIcon />
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
                    <Text color="tomato">{errors.temps}</Text>
                }
            </div>
            {
                !!errors?.other &&
                <Text color="tomato">{errors.other}</Text>
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
