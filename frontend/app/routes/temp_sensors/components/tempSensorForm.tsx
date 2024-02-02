import type {TempSensor} from "~/routes/temp_sensors/types";
import {useEffect, useRef, useState} from 'react';
import type {Room} from "~/routes/rooms/types";
import {Form, useActionData} from "@remix-run/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import {routes} from "~/routes";
import {capitalizeAndRemoveUnderscore, formatNumber} from "~/utils/formattingUtils";
import type {FormErrors} from "~/utils/types";
import {Input} from "~/components/ui/input";
import {Badge} from "~/components/ui/badge";
import {RadioGroup, RadioGroupItem} from "~/components/ui/radio-group";
import {Button} from "~/components/ui/button";
import {useNavigation} from "react-router";
import {Label} from "~/components/ui/label";

export interface TempSensorFormProps {
    rooms: Room[]
    sensor?: TempSensor
}

const TempSensorForm = ({rooms, sensor}: TempSensorFormProps) => {
    const actionData = useActionData<FormErrors<TempSensor>>();

    const navigation = useNavigation();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(sensor);

    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<FormErrors<TempSensor> | null>(null);

    useEffect(() => {
        if (actionData && !sensor && !actionData.id) {
            setErrors(actionData);
        } else if (actionData && sensor?.id === actionData.id) {
            setErrors(actionData);
        } else {
            setErrors(null);
        }
    }, [actionData]);

    useEffect(() => {
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [navigation]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.TEMP_SENSORS.ROOT}>
            <div>
                <label className="font-bold" htmlFor="id">ID</label>
                {
                    !sensor ?
                    <>
                        <Input name="id"/>
                        <>
                            {
                                !!errors?.id &&
                                <p color="tomato">{errors.id}</p>
                            }
                        </>
                    </>
                        :
                        <div>
                            <p>{sensor.id}</p>
                            {
                                sensor.battery_level !== null &&
                                <Badge
                                    className="text-left"
                                >
                                    {`${formatNumber(sensor.battery_level, 0, 0)} %`}
                                </Badge>
                            }
                        </div>

                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold" htmlFor="room_id">Room</label>
                {
                    sensor === undefined ?
                        <>
                            <RadioGroup name="room_id">
                                <div className="flex flex-col">
                                    {rooms.map((room) => {
                                        return <div className="flex items-center space-x-2" key={room.id}>
                                            <RadioGroupItem
                                                key={room?.id}
                                                id={room.id}
                                                value={room.id}
                                            />
                                            <Label htmlFor={room.id}>{capitalizeAndRemoveUnderscore(room.name)}</Label>
                                        </div>;
                                    })}
                                </div>
                            </RadioGroup>
                            {
                                !!errors?.room_id &&
                                <p color="tomato">{errors.room_id}</p>
                            }
                        </>
                        :
                        <p>{rooms.find(room => room.id == sensor?.room_id)?.name ?? 'Unknown room'}</p>
                }
            </div>
            <div className="mt-1">
                {
                    isNew &&
                    <Button className="mr-1" type="submit" name="intent" value={'create'}
                            disabled={isCreating}>{"Add"}</Button>
                }
                {
                    !isNew &&
                    <>
                        <input type="hidden" name="id" value={sensor?.id}/>
                        <Button variant="outline" type="submit" name="intent" value="delete"
                                disabled={isDeleting}>{isDeleting ? 'Deleting...' : 'Delete'}</Button>
                    </>
                }
            </div>

        </Form>
    );
};

export default TempSensorForm;
