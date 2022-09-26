import React, {useEffect, useState} from 'react';
import {Form, useActionData, useTransition} from "remix";
import {routes} from "~/routes";
import {Plug} from "~/routes/plugs/types/types";
import {PlugFormErrors} from "~/routes/plugs";

export interface PlugFormProps {
    plug?: Plug
}

const PlugForm = ({plug}: PlugFormProps) => {
    const actionData = useActionData<PlugFormErrors>();
    const transition = useTransition()

    const [errors, setErrors] = useState<PlugFormErrors | null>(null);

    useEffect(() => {
        if (actionData && !plug && !actionData.id) {
            setErrors(actionData)
        } else if (actionData && plug?.id === actionData.id) {
            setErrors(actionData)
        } else {
            setErrors(null)
        }
    }, [actionData])

    useEffect(() => {
        console.log(transition)
    }, [transition])

    const renderErrors = (errors: PlugFormErrors) => {
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

    return (
        <Form className="border-4 my-2 p-2" method="post" action={routes.PLUGS.ROOT}>
            <input hidden readOnly name="id" value={plug?.id}/>
            <div>
                <label className="font-bold">Name</label>
                <input name="name" defaultValue={plug?.name}/>
            </div>
            <div>
                <label className="font-bold">IP address</label>
                <input name="ip" defaultValue={plug?.ip}/>
            </div>

            <button className="border-4 rounded-md" type="submit" disabled>{plug ? "Edit" : "Create"}</button>
            {
                errors &&
                renderErrors(errors)
            }
        </Form>
    );
};

export default PlugForm;