import {useEffect, useRef, useState} from 'react';
import {routes} from "~/routes";
import type {Plug} from "~/routes/plugs/types";
import type {PlugFormErrors} from "~/routes/plugs";
import {Form, useActionData} from "@remix-run/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {Room} from '~/routes/rooms/types';
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {Input} from '~/components/ui/input';
import {RadioGroup, RadioGroupItem} from "~/components/ui/radio-group";
import {Checkbox} from "~/components/ui/checkbox";
import {Button} from "~/components/ui/button";
import {useNavigation} from "react-router";

export interface PlugFormProps {
    plug?: Plug
    rooms: Room[]
}

const PlugForm = ({plug, rooms}: PlugFormProps) => {
    const actionData = useActionData<PlugFormErrors>();

    const navigation = useNavigation();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(plug);

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
    }, [navigation]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.PLUGS.ROOT}>
            <input hidden readOnly name="id" value={plug?.id}/>
            <div>
                <label className="font-bold" htmlFor="name">Name</label>
                <Input name="name" defaultValue={plug?.name}/>
                {
                    !!errors?.name &&
                    <p color="tomato">{errors.name}</p>
                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold" htmlFor="priceLevel">Room</label>
                <RadioGroup defaultValue={plug?.room_id} name="priceLevel">
                    <div className="flex flex-row">
                        {rooms.map((room) => {
                            return <RadioGroupItem
                                key={plug?.id + room.id}
                                id="room_id"
                                checked={plug?.room_id === room.id}
                                value={room.id}>
                                {capitalizeAndRemoveUnderscore(room.name)}
                            </RadioGroupItem>;
                        })}
                    </div>
                </RadioGroup>
                {
                    !!errors?.room_id &&
                    <p color="tomato">{errors.room_id}</p>
                }
            </div>
            <div>
                <label className="font-bold" htmlFor="ip">IP address</label>
                <Input name="ip" defaultValue={plug?.ip}/>
                {
                    !!errors?.ip &&
                    <p color="tomato">{errors.ip}</p>
                }
            </div>
            <div>
                <label className="font-bold" htmlFor="username">Username</label>
                <Input name="username" defaultValue={plug?.username}/>
                {
                    !!errors?.username &&
                    <p color="tomato">{errors.username}</p>
                }
            </div>
            <div>
                <label className="font-bold" htmlFor="password">Password</label>
                <Input name="password" defaultValue={plug?.password}/>
                {
                    !!errors?.password &&
                    <p color="tomato">{errors.password}</p>
                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold" htmlFor="scheduled">Scheduled</label>
                    <Checkbox
                        id="scheduled"
                        name="scheduled"
                        defaultChecked={plug?.scheduled}
                    />
                {
                    !!errors?.scheduled &&
                    <p color="tomato">{errors.scheduled}</p>
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
