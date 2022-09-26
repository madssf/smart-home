import React, {useEffect, useRef, useState} from 'react';
import {PRICE_LEVELS, Schedule, TimeWindow, WEEKDAYS} from "~/routes/schedules/types/types";
import TimeForm from "~/routes/schedules/components/timeForm";
import {routes} from "~/routes";
import {ScheduleFormErrors} from "~/routes/schedules";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {Button, Text} from "@chakra-ui/react";

export interface ScheduleFormProps {
    schedule?: Schedule
}

const ScheduleForm = ({schedule}: ScheduleFormProps) => {
    const actionData = useActionData<ScheduleFormErrors>();
    const transition = useTransition()
    const isCreating = transition.submission?.formData.get("intent") === "create" && (transition.submission?.formData.get('id') ?? undefined) === schedule?.id;
    const isUpdating = transition.submission?.formData.get("intent") === "update" && (transition.submission?.formData.get('id') ?? undefined) === schedule?.id;
    const isDeleting = transition.submission?.formData.get("intent") === "delete" && (transition.submission?.formData.get('id') ?? undefined) === schedule?.id;
    const isNew = !schedule
    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<ScheduleFormErrors | null>(null);

    useEffect(() => {
        if (actionData && !schedule && !actionData.id) {
            setErrors(actionData)
        } else if (actionData && schedule?.id === actionData.id) {
            setErrors(actionData)
        } else {
            setErrors(null)
        }
    }, [actionData])

    useEffect(() => {
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
    }, [transition])

    const [hoursList, setHoursList] = useState(schedule?.hours ?? []);

    const renderErrors = (errors: ScheduleFormErrors) => {
        const { id, ...rest} = errors
        return (
            <ul>
                {
                    Object.values(rest).filter((value) => value).map((error) => {
                        return <li>{error}</li>
                    })
                }
            </ul>
        )
    }

    const handleRemoveTimeWindow = (toRemove: TimeWindow) => {
        setHoursList((prev) => prev.filter((existing) => {
            return existing.from !== toRemove.from && existing.to !== toRemove.to
        }))
    }

    const addTimeWindow = () => {
        setHoursList((prev) => prev.concat([{
            from: "01:00",
            to: "02:00",
        }]))
    }




    return (
        <Form ref={formRef} method="post" action={routes.SCHEDULES.ROOT}>
            <input hidden readOnly name="id" value={schedule?.id}/>
            <div>
                <label className="font-bold">Price level</label>
                <select name="priceLevel" defaultValue={schedule?.priceLevel}>
                    {PRICE_LEVELS.map((priceLevel) => {
                        return <option key={priceLevel} value={priceLevel} label={priceLevel}/>
                    })}
                </select>
                {
                    !!errors?.priceLevel &&
                    <Text color="tomato">{errors.priceLevel}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Weekdays</label>
                {WEEKDAYS.map((day) => {
                    return <label key={day}><input type="checkbox" id={day} name="days" value={day}
                                                   defaultChecked={schedule?.days.includes(day)}/>{day}</label>
                })}
                {
                    !!errors?.days &&
                    <Text color="tomato">{errors.days}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Time windows</label>
            {
                <div className="ml-4">
                    {
                        (hoursList).map((window, i) => {
                            return <TimeForm key={i} window={window} handleRemove={() => handleRemoveTimeWindow(window)}/>
                        })
                    }
                    <Button size="sm" type="button" onClick={() => addTimeWindow()}>Add</Button>
                </div>
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
                <Button type="submit" name="intent" value={isNew ? 'create' : 'update'}
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