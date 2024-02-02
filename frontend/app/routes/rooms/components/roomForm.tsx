import type {Room} from "~/routes/rooms/types";
import {useEffect, useRef, useState} from 'react';
import {Form, useActionData} from "@remix-run/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {RoomFormErrors} from "~/routes/rooms";
import {routes} from "~/routes";
import {Input} from "~/components/ui/input";
import {Button} from "~/components/ui/button";
import {useNavigation} from "react-router";

export interface RoomFormProps {
    room?: Room
}


const RoomForm = ({room}: RoomFormProps) => {
    const actionData = useActionData<RoomFormErrors>();

    const navigation = useNavigation();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(room);
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
    }, [navigation]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.ROOMS.ROOT}>
            <div
                className="flex flex-row gap-1"
            >
                <input hidden readOnly name="id" value={room?.id}/>
                <div>
                    <label className="font-bold" htmlFor="name">Name</label>
                    <Input name="name" defaultValue={room?.name}/>
                    {
                        !!errors?.name &&
                        <p color="tomato">{errors.name}</p>
                    }
                    {room?.id &&
                        <p className="text-xs text-gray-500 dark:text-gray-200">{room.id}</p>
                    }
                </div>
                <div>
                    <label className="font-bold" htmlFor="min_temp">Min temp</label>
                    <div
                        className="flex items-center"
                    >
                        <Input
                            type="number"
                            min="1"
                            max="100"
                            step="1"
                            name="min_temp"
                            defaultValue={room?.min_temp ?? undefined}
                        />
                        <span
                            className="ml-2 text-gray-600 dark:text-gray-400"
                        >
                            °C
                        </span>
                    </div>
                </div>
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
