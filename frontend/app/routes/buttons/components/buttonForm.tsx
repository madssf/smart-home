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
import {Label} from "~/components/ui/label";

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
                <Label htmlFor="name">Name</Label>
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
                <p className="font-bold">Plugs</p>
                <div className="flex flex-col space-y-3 my-2">
                    {plugs.map((plug) => {
                        return <div
                            key={button?.id + plug.id}
                            className="flex flex-row space-x-1"
                        >
                            <Checkbox
                                className="mr-1"
                                id={plug.id}
                                name="plug_id"
                                value={plug.id}
                                defaultChecked={button?.plug_ids.includes(plug.id)}>
                            </Checkbox>
                            <Label htmlFor={plug.id}>{plug.name}</Label>
                        </div>;
                    })}
                </div>
                {
                    !!errors?.plug_ids &&
                    <p color="tomato">{errors.plug_ids}</p>
                }
            </div>
            <div>
                <Label htmlFor="ip">IP address</Label>
                <Input name="ip" defaultValue={button?.ip}/>
                {
                    !!errors?.ip &&
                    <p color="tomato">{errors.ip}</p>
                }
            </div>
            <div>
                <Label htmlFor="username">Username</Label>
                <Input name="username" defaultValue={button?.username}/>
                {
                    !!errors?.username &&
                    <p color="tomato">{errors.username}</p>
                }
            </div>
            <div>
                <Label htmlFor="password">Password</Label>
                <Input name="password" defaultValue={button?.password}/>
                {
                    !!errors?.password &&
                    <p color="tomato">{errors.password}</p>
                }
            </div>
            <div className="mt-3">
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
