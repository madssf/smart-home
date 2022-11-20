import type {TempSensor} from "~/routes/temp_sensors/types";
import React, {useEffect, useRef, useState} from 'react';
import type {Room} from "~/routes/rooms/types";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import {routes} from "~/routes";
import {Button, Input, Radio, RadioGroup, Stack, Text} from "@chakra-ui/react";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import type {FormErrors} from "~/utils/types";

export interface TempSensorFormProps {
    rooms: Room[]
    sensor?: TempSensor
}

const TempSensorForm = ({rooms, sensor}: TempSensorFormProps) => {
    const actionData = useActionData<FormErrors<TempSensor>>();
    const transition = useTransition();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(transition, sensor);

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
    }, [transition]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.TEMP_SENSORS.ROOT}>
            <div>
                <label className="font-bold">ID</label>
                <Input name="id" defaultValue={sensor?.id}/>
                {
                    !!errors?.id &&
                    <Text color="tomato">{errors.id}</Text>
                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold">Room</label>
                <RadioGroup defaultValue={sensor?.room_id} name="actionType">
                    <Stack direction="row">
                        {rooms.map((room) => {
                            return <Radio
                                key={room?.id + sensor?.id}
                                id="room_id"
                                name="room_id"
                                checked={sensor?.room_id === room.id}
                                value={room.id}>
                                {capitalizeAndRemoveUnderscore(room.name)}
                            </Radio>;
                        })}
                    </Stack>
                </RadioGroup>
                {
                    !!errors?.room_id &&
                    <Text color="tomato">{errors.room_id}</Text>
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
                    <Button variant="outline" type="submit" name="intent" value="delete"
                            disabled={isDeleting}>{isDeleting ? 'Deleting...' : 'Delete'}</Button>

                }
            </div>

        </Form>
    );
};

export default TempSensorForm;
