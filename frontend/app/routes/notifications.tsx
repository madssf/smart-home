import React from 'react';
import type {ActionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import {piTriggerRefresh} from "~/utils/piHooks";
import {routes} from "~/routes";
import {validateNonEmptyString, validatePositiveNonZeroInteger} from "~/utils/validation";
import type {FormErrors} from "~/utils/types";
import type {NotificationSettings} from "~/routes/notifications/types";
import {getNotificationSettings, upsertNotificationSettings} from "~/routes/notifications/notifications.server";
import {Button, Heading, Input, InputGroup, InputRightAddon, Text} from "@chakra-ui/react";
import {Form, useActionData, useLoaderData, useTransition} from "@remix-run/react";

export interface SettingsProps {
    max_consumption: number | null
}

interface ResponseData {
    settings: NotificationSettings | null,
}

export async function action({request}: ActionArgs) {

    const body = await request.formData();

    const max_consumption = body.get("max_consumption")?.toString();
    const max_consumption_timeout_minutes = body.get("max_consumption_timeout_minutes")?.toString();
    const ntfy_topic = body.get("ntfy_topic")?.toString();

    const validated = {
        max_consumption: validatePositiveNonZeroInteger(max_consumption),
        max_consumption_timeout_minutes: validatePositiveNonZeroInteger(max_consumption_timeout_minutes),
        ntfy_topic: validateNonEmptyString(ntfy_topic),
    };

    if (!validated.max_consumption.valid || !validated.max_consumption_timeout_minutes.valid || !validated.ntfy_topic.valid) {
        return json<FormErrors<NotificationSettings>>(
            {
                max_consumption: !validated.max_consumption.valid ? validated.max_consumption.error : undefined,
                max_consumption_timeout_minutes: !validated.max_consumption_timeout_minutes.valid ?
                    validated.max_consumption_timeout_minutes.error : undefined,
                ntfy_topic: !validated.ntfy_topic.valid ? validated.ntfy_topic.error : undefined,
            },
        );
    }

    const document: NotificationSettings = {
        max_consumption: validated.max_consumption.data,
        max_consumption_timeout_minutes: validated.max_consumption_timeout_minutes.data,
        ntfy_topic: validated.ntfy_topic.data,
    };

    await upsertNotificationSettings(document);
    await piTriggerRefresh();
    return redirect(routes.NOTIFICATIONS.ROOT);
}

export const loader: LoaderFunction = async () => {

    const settings = await getNotificationSettings();

    return json<ResponseData>({
        settings,
    });

};

export const handle = {hydrate: true};

const Notifications = () => {

    const loaderData = useLoaderData<ResponseData>();
    const settings = loaderData.settings;
    const actionData = useActionData<FormErrors<NotificationSettings>>();
    const transition = useTransition();

    return (
        <div>
            <Heading className="pb-4">Settings</Heading>
            <Form className="mb-2" method="post" action={routes.NOTIFICATIONS.ROOT}>
                <label className="font-bold">Max consumption</label>
                <InputGroup>
                    <Input
                        type="number"
                        min="1"
                        step="1"
                        name="max_consumption"
                        defaultValue={settings?.max_consumption ?? undefined}
                    />
                    <InputRightAddon children="W" />
                </InputGroup>
                {
                    !!actionData?.max_consumption &&
                    <Text color="tomato">{actionData?.max_consumption}</Text>
                }
                <label className="font-bold">Max consumption timeout</label>
                <InputGroup>
                    <Input
                        type="number"
                        min="1"
                        step="1"
                        name="max_consumption_timeout_minutes"
                        defaultValue={settings?.max_consumption_timeout_minutes ?? undefined}
                    />
                    <InputRightAddon children="minutes" />
                </InputGroup>
                {
                    !!actionData?.max_consumption_timeout_minutes &&
                    <Text color="tomato">{actionData?.max_consumption_timeout_minutes}</Text>
                }
                <label className="font-bold">NTFY topic</label>
                <Input name="ntfy_topic" defaultValue={settings?.ntfy_topic ?? undefined}/>
                {
                    !!actionData?.ntfy_topic &&
                    <Text color="tomato">{actionData?.ntfy_topic}</Text>
                }
                <Button
                    className="mr-1 mt-2"
                    type="submit"
                    disabled={transition.state === 'submitting'}
                >
                    {transition.state === 'submitting' ? "Updating" : "Update"}
                </Button>
            </Form>

        </div>
    );
};

export default Notifications;
