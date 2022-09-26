import React, {useEffect, useRef, useState} from 'react';
import {routes} from "~/routes";
import {Plug} from "~/routes/plugs/types/types";
import {PlugFormErrors} from "~/routes/plugs";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {Input} from "@chakra-ui/input";
import {Box, Button, Text} from "@chakra-ui/react";

export interface PlugFormProps {
    plug?: Plug
}

const PlugForm = ({plug}: PlugFormProps) => {
    const actionData = useActionData<PlugFormErrors>();
    const transition = useTransition()
    const isCreating = transition.submission?.formData.get("intent") === "create" && (transition.submission?.formData.get('id') ?? undefined) === plug?.id;
    const isUpdating = transition.submission?.formData.get("intent") === "update" && (transition.submission?.formData.get('id') ?? undefined) === plug?.id;
    const isDeleting = transition.submission?.formData.get("intent") === "delete" && (transition.submission?.formData.get('id') ?? undefined) === plug?.id;
    const isNew = !plug
    const formRef = useRef<HTMLFormElement>(null);

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
        if (!isCreating || !isUpdating) {
            formRef.current?.reset();
        }
    }, [transition])

    return (
        <Box p={2}>
            <Form ref={formRef} method="post" action={routes.PLUGS.ROOT}>
                <input hidden readOnly name="id" value={plug?.id}/>
                <div>
                    <label className="font-bold">Name</label>
                    <Input name="name" defaultValue={plug?.name}/>
                    {
                        !!errors?.name &&
                        <Text color="tomato">{errors.name}</Text>
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
                    <Button type="submit" name="intent" value={isNew ? 'create' : 'update'}
                            disabled={isCreating || isUpdating}>{isNew ? "Add" : "Update"}</Button>
                    {
                        !isNew &&
                        <Button variant="outline" type="submit" name="intent" value="delete"
                                disabled={isDeleting}>{isDeleting ? 'Deleting...' : 'Delete'}</Button>

                    }
                </div>

            </Form>
        </Box>
    );
};

export default PlugForm;