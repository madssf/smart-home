import React, {useEffect, useState} from 'react';
import {PRICE_LEVELS, Schedule, TimeWindow, WEEKDAYS} from "~/routes/schedules/types/types";
import TimeForm from "~/routes/schedules/components/timeForm";
import {Form, useActionData} from "remix";
import {routes} from "~/routes";
import {ScheduleFormErrors} from "~/routes/schedules";

export interface ScheduleFormProps {
    schedule?: Schedule
}

const ScheduleForm = ({schedule}: ScheduleFormProps) => {
    const actionData = useActionData<ScheduleFormErrors>();

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
        <Form className="border-4 my-2 p-2" method="post" action={routes.SCHEDULES.ROOT}>
            <input hidden readOnly name="id" value={schedule?.id}/>
            <div>
                <label className="font-bold">Price level</label>
                <select name="priceLevel" defaultValue={schedule?.priceLevel}>
                    {PRICE_LEVELS.map((priceLevel) => {
                        return <option key={priceLevel} value={priceLevel} label={priceLevel}/>
                    })}
                </select>
            </div>
            <div>
                <label className="font-bold">Weekdays</label>
                {WEEKDAYS.map((day) => {
                    return <label key={day}><input type="checkbox" id={day} name="days" value={day}
                                                   defaultChecked={schedule?.days.includes(day)}/>{day}</label>
                })}
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
                    <button type="button" onClick={() => addTimeWindow()}>Add</button>
                </div>
            }
            </div>

            <button type="submit">{schedule ? "Edit" : "Create"}</button>
            {
                errors &&
                    renderErrors(errors)
            }
        </Form>
    );
};

export default ScheduleForm;