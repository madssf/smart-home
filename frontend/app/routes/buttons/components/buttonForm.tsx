import {useEffect, useRef, useState} from 'react';
import {routes} from "~/routes";
import type {Plug} from "~/routes/plugs/types";
import {Form, useActionData} from "@remix-run/react";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";
import type {ButtonType} from "~/routes/buttons/types";
import type {ButtonFormErrors} from "~/routes/buttons";
import {Input} from "~/components/ui/input";
import {Checkbox} from "~/components/ui/checkbox";
import {Button} from "~/components/ui/button";
import {useNavigation} from "react-router";

export interface ButtonFormProps {
    button?: ButtonType
    plugs: Plug[]
}

const ButtonForm = ({button, plugs}: ButtonFormProps) => {
    const actionData = useActionData<ButtonFormErrors>();

    const navigation = useNavigation();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(button);

    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<ButtonFormErrors | null>(null);

    useEffect(() => {
        if (actionData && !button && !actionData.id) {
            setErrors(actionData);
        } else if (actionData && button?.id === actionData.id) {
            setErrors(actionData);
        } else {
            setErrors(null);
        }
    }, [actionData, button]);

    useEffect(() => {
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [navigation]);

    return (
        <Form className="mb-2" ref={formRef} method="post" action={routes.BUTTONS.ROOT}>
            <input hidden readOnly name="id" value={button?.id}/>
            <div>
                <label className="font-bold">Name</label>
                <Input name="name" defaultValue={button?.name}/>
                {
                    !!errors?.name &&
                    <p color="tomato">{errors.name}</p>
                }
            </div>
            {button?.id &&
                <p className="text-sm text-gray-400">{button.id}</p>
            }
            <div className="flex flex-col">
                <label className="font-bold">Plugs</label>
                <div className="flex">
                    {plugs.map((plug) => {
                        return <Checkbox
                            key={button?.id + plug.id}
                            className="mr-1"
                            id={plug.id}
                            name="plug_id"
                            value={plug.id}
                            defaultChecked={button?.plug_ids.includes(plug.id)}>
                            {plug.name}
                        </Checkbox>;
                    })}
                </div>
                {
                    !!errors?.plug_ids &&
                    <p color="tomato">{errors.plug_ids}</p>
                }
            </div>
            <div>
                <label className="font-bold">IP address</label>
                <Input name="ip" defaultValue={button?.ip}/>
                {
                    !!errors?.ip &&
                    <p color="tomato">{errors.ip}</p>
                }
            </div>
            <div>
                <label className="font-bold">Username</label>
                <Input name="username" defaultValue={button?.username}/>
                {
                    !!errors?.username &&
                    <p color="tomato">{errors.username}</p>
                }
            </div>
            <div>
                <label className="font-bold">Password</label>
                <Input name="password" defaultValue={button?.password}/>
                {
                    !!errors?.password &&
                    <p color="tomato">{errors.password}</p>
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

export default ButtonForm;