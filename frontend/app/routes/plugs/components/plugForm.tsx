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
import {Label} from '~/components/ui/label';

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
        <Form className="mb-2 flex flex-col gap-2" ref={formRef} method="post" action={routes.PLUGS.ROOT}>
            <input hidden readOnly name="id" value={plug?.id}/>
            <div>
                <Label htmlFor="name">Name</Label>
                <Input name="name" defaultValue={plug?.name}/>
                {
                    !!errors?.name &&
                    <p color="tomato">{errors.name}</p>
                }
            </div>
            <div className="flex flex-col">
                <Label htmlFor="room_id">Room</Label>
                <RadioGroup defaultValue={plug?.room_id} name="room_id">
                    <div className="flex flex-col space-y-3 my-2">
                        {rooms.map((room) => {
                            return <div className="flex items-center space-x-2" key={room.id}>
                                <RadioGroupItem
                                    key={plug?.id + room.id}
                                    id={room.id}
                                    value={room.id}
                                />
                                <Label htmlFor={room.id}>{capitalizeAndRemoveUnderscore(room.name)}</Label>
                            </div>
                        })}
                    </div>
                </RadioGroup>
                {
                    !!errors?.room_id &&
                    <p color="tomato">{errors.room_id}</p>
                }
            </div>
            <div>
                <Label htmlFor="ip">IP address</Label>
                <Input name="ip" defaultValue={plug?.ip}/>
                {
                    !!errors?.ip &&
                    <p color="tomato">{errors.ip}</p>
                }
            </div>
            <div>
                <Label htmlFor="username">Username</Label>
                <Input name="username" defaultValue={plug?.username}/>
                {
                    !!errors?.username &&
                    <p color="tomato">{errors.username}</p>
                }
            </div>
            <div>
                <Label htmlFor="password">Password</Label>
                <Input name="password" defaultValue={plug?.password}/>
                {
                    !!errors?.password &&
                    <p color="tomato">{errors.password}</p>
                }
            </div>
            <div className="flex flex-row gap-2 items-center my-2">
                <Label htmlFor="scheduled">Scheduled</Label>
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
