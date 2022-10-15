import React, {useEffect, useRef, useState} from 'react';
import type {Schedule, TimeWindow} from "~/routes/schedules/types/types";
import {PRICE_LEVELS, WEEKDAYS} from "~/routes/schedules/types/types";
import TimeForm from "~/routes/schedules/components/timeForm";
import {routes} from "~/routes";
import type {ScheduleFormErrors} from "~/routes/schedules";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {Button, Checkbox, Radio, RadioGroup, Stack, Text} from "@chakra-ui/react";
import {capitalizeAndRemoveUnderscore} from '~/utils/formattingUtils';
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";

export interface ScheduleFormProps {
    schedule?: Schedule
}

const ScheduleForm = ({schedule}: ScheduleFormProps) => {
    const actionData = useActionData<ScheduleFormErrors>();
    const transition = useTransition();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(transition, schedule);

    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<ScheduleFormErrors | null>(null);

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
        setHoursList(schedule?.hours ?? []);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [transition]);

    const [hoursList, setHoursList] = useState(schedule?.hours ?? []);

    const handleRemoveTimeWindow = (toRemove: TimeWindow) => {
        setHoursList((prev) => prev.filter((existing) => {
            return existing.from !== toRemove.from && existing.to !== toRemove.to;
        }));
    };

    const addTimeWindow = () => {
        setHoursList((prev) => prev.concat([{
            from: "01:00",
            to: "02:00",
        }]));
    };

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.SCHEDULES.ROOT}>
            <input hidden readOnly name="id" value={schedule?.id}/>
            <div className="flex flex-col">
                <label className="font-bold">Price level</label>
                <RadioGroup defaultValue={schedule?.priceLevel} name="priceLevel">
                    <Stack direction="row">
                        {PRICE_LEVELS.map((priceLevel) => {
                            return <Radio
                                key={schedule?.id + priceLevel}
                                id="priceLevel"
                                name="priceLevel"
                                checked={schedule?.priceLevel === priceLevel}
                                value={priceLevel}>
                                {capitalizeAndRemoveUnderscore(priceLevel)}
                            </Radio>;
                        })}
                    </Stack>
                </RadioGroup>
                {
                    !!errors?.priceLevel &&
                    <Text color="tomato">{errors.priceLevel}</Text>
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
                        defaultChecked={schedule?.days.includes(day)}>
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
                <>
                <div className="ml-2 mb-1">
                    {
                        (hoursList).map((window, i) => {
                            return <TimeForm key={i} window={window} handleRemove={() => handleRemoveTimeWindow(window)}/>;
                        })
                    }
                </div>
                <Button className="mb-1" size="sm" type="button" onClick={() => addTimeWindow()}>Add time window</Button>
                </>
            }
                {
                    !!errors?.hours &&
                    <Text color="tomato">{errors.hours}</Text>
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
