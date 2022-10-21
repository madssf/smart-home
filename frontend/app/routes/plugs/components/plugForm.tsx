import React, {useEffect, useRef, useState} from 'react';
import {routes} from "~/routes";
import type {Plug} from "~/routes/plugs/types";
import type {PlugFormErrors} from "~/routes/plugs";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {Input} from "@chakra-ui/input";
import {Button, Radio, RadioGroup, Stack, Text} from "@chakra-ui/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {Room} from '~/routes/rooms/types';
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";

export interface PlugFormProps {
    plug?: Plug
    rooms: Room[]
}

const PlugForm = ({plug, rooms}: PlugFormProps) => {
    const actionData = useActionData<PlugFormErrors>();
    const transition = useTransition();

    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(transition, plug);

    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<PlugFormErrors | null>(null);

    useEffect(() => {
        if (actionData && !plug && !actionData.id) {
            setErrors(actionData);
        } else if (actionData && plug?.id === actionData.id) {
            setErrors(actionData);
        } else {
            setErrors(null);
        }
    }, [actionData, plug]);

    useEffect(() => {
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [transition]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.PLUGS.ROOT}>
            <input hidden readOnly name="id" value={plug?.id}/>
            <div>
                <label className="font-bold">Name</label>
                <Input name="name" defaultValue={plug?.name}/>
                {
                    !!errors?.name &&
                    <Text color="tomato">{errors.name}</Text>
                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold">Room</label>
                <RadioGroup defaultValue={plug?.room_id} name="priceLevel">
                    <Stack direction="row">
                        {rooms.map((room) => {
                            return <Radio
                                key={plug?.id + room.id}
                                id="room_id"
                                name="room_id"
                                checked={plug?.room_id === room.id}
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
            <div>
                <label className="font-bold">IP address</label>
                <Input name="ip" defaultValue={plug?.ip}/>
                {
                    !!errors?.ip &&
                    <Text color="tomato">{errors.ip}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Username</label>
                <Input name="username" defaultValue={plug?.username}/>
                {
                    !!errors?.username &&
                    <Text color="tomato">{errors.username}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Password</label>
                <Input name="password" defaultValue={plug?.password}/>
                {
                    !!errors?.password &&
                    <Text color="tomato">{errors.password}</Text>
                }
            </div>
            <div className="mt-1">
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

export default PlugForm;
