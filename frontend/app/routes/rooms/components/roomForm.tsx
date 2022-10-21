import type {Room} from "~/routes/rooms/types";
import React, {useEffect, useRef, useState} from 'react';
import {Form, useActionData, useTransition} from "@remix-run/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {RoomFormErrors} from "~/routes/rooms";
import {routes} from "~/routes";
import {Input} from "@chakra-ui/input";
import {Button, Text} from "@chakra-ui/react";

export interface RoomFormProps {
    room?: Room
}


const RoomForm = ({room}: RoomFormProps) => {
    const actionData = useActionData<RoomFormErrors>();
    const transition = useTransition();

    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(transition, room);
    const formRef = useRef<HTMLFormElement>(null);
    const [errors, setErrors] = useState<RoomFormErrors | null>(null);

    useEffect(() => {
        if (actionData && !room && !actionData.id) {
            setErrors(actionData);
        } else if (actionData && room?.id === actionData.id) {
            setErrors(actionData);
        } else {
            setErrors(null);
        }
    }, [actionData, room]);

    useEffect(() => {
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [transition]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.ROOMS.ROOT}>
            <input hidden readOnly name="id" value={room?.id}/>
            <div>
                <label className="font-bold">Name</label>
                <Input name="name" defaultValue={room?.name}/>
                {
                    !!errors?.name &&
                    <Text color="tomato">{errors.name}</Text>
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

export default RoomForm;