import React, {useEffect, useRef, useState} from 'react';
import {routes} from "~/routes";
import type {Plug} from "~/routes/plugs/types/types";
import {Form, useActionData, useTransition} from "@remix-run/react";
import {Button, Checkbox, Radio, RadioGroup, Stack, Text} from "@chakra-ui/react";
import type {TempAction} from "~/routes/temp_actions/types";
import {ActionType} from "~/routes/temp_actions/types";
import type {TempActionErrors} from "~/routes/temp_actions";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import DatePicker from "~/components/datePicker";
import {useSubmissionStatus} from "~/hooks/useSubmissionStatus";

export interface TempActionFormProps {
    tempAction?: TempAction;
    plugs: Plug[]
}

const TempActionForm = ({tempAction, plugs}: TempActionFormProps) => {
    const actionData = useActionData<TempActionErrors>();
    const transition = useTransition();
    const {isCreating, isDeleting, isUpdating, isNew} = useSubmissionStatus(transition, tempAction);

    const formRef = useRef<HTMLFormElement>(null);

    const [errors, setErrors] = useState<TempActionErrors | null>(null);

    useEffect(() => {
        if (actionData && !tempAction && !actionData.id) {
            setErrors(actionData);
        } else if (actionData && tempAction?.id === actionData.id) {
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
        <Form className="mb-2" ref={formRef} method="post" action={routes.TEMP_ACTIONS.ROOT}>
            <input hidden readOnly name="id" value={tempAction?.id}/>
            <div className="flex flex-col">
                <label className="font-bold">Action</label>
                <RadioGroup defaultValue={tempAction?.action_type} name="actionType">
                    <Stack direction="row">
                        {Object.values(ActionType).map((actionType) => {
                            return <Radio
                                key={tempAction?.id + actionType}
                                id="actionType"
                                name="actionType"
                                checked={tempAction?.action_type === actionType}
                                value={actionType}>
                                {capitalizeAndRemoveUnderscore(actionType)}
                            </Radio>;
                        })}
                    </Stack>
                </RadioGroup>
                {
                    !!errors?.action_type &&
                    <Text color="tomato">{errors.action_type}</Text>
                }
            </div>
            <div>
                <label className="font-bold">Expires at</label>
                <DatePicker name={'expiresAt'} defaultValue={tempAction?.expires_at} />
                {
                    !!errors?.expires_at &&
                    <Text color="tomato">{errors.expires_at}</Text>
                }
            </div>
            <div className="flex flex-col">
                <label className="font-bold">Plugs</label>
                <div className="flex">
                    {plugs.map((plug) => {
                        return <Checkbox
                            key={tempAction?.id + plug.id}
                            size="sm"
                            className="mr-1"
                            id={plug.id}
                            name="plugIds"
                            value={plug.id}
                            defaultChecked={tempAction?.plug_ids.includes(plug.id)}>
                            {capitalizeAndRemoveUnderscore(plug.name)}
                        </Checkbox>;
                    })}
                </div>
                {
                    !!errors?.plug_ids &&
                    <Text color="tomato">{errors.plug_ids}</Text>
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

export default TempActionForm;
