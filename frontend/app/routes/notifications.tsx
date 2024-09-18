import type {ActionFunctionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import {piTriggerRefresh} from "~/utils/piHooks";
import {routes} from "~/routes";
import {validateNonEmptyString, validatePositiveNonZeroInteger} from "~/utils/validation";
import type {FormErrors} from "~/utils/types";
import type {NotificationSettings} from "~/routes/notifications/types";
import {getNotificationSettings, upsertNotificationSettings} from "~/routes/notifications/notifications.server";
import {Form, useActionData, useLoaderData} from "@remix-run/react";
import {Input} from "~/components/ui/input";
import {Button} from "~/components/ui/button";
import {useNavigation} from "react-router";
import {Label} from "~/components/ui/label";

export interface SettingsProps {
    max_consumption: number | null
}

interface ResponseData {
    settings: NotificationSettings | null,
}

export async function action({request}: ActionFunctionArgs) {

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
    const navigation = useNavigation();

    return (
        <div>
            <h1 className="pb-4">Notifications</h1>
            <Form className="mb-2" method="post" action={routes.NOTIFICATIONS.ROOT}>
                <Label htmlFor="max_consumption">Max consumption</Label>
                <div
                    className="flex items-center"
                >
                    <Input
                        type="number"
                        style={{maxWidth: "150px"}}
                        pattern="[0-9]*"
                        min="1"
                        step="1"
                        name="max_consumption"
                        defaultValue={settings?.max_consumption ?? undefined}
                    />
                    <span
                        className="ml-2 text-gray-600 dark:text-gray-400"
                    >
                        Watt
                    </span>
                </div>
                {
                    !!actionData?.max_consumption &&
                    <p color="tomato">{actionData?.max_consumption}</p>
                }
                <Label htmlFor="max_consumption_timeout_minutes">Max consumption timeout</Label>
                <div
                    className="flex items-center"
                >
                    <Input
                        type="number"
                        style={{maxWidth: "150px"}}
                        pattern="[0-9]*"
                        min="1"
                        step="1"
                        name="max_consumption_timeout_minutes"
                        defaultValue={settings?.max_consumption_timeout_minutes ?? undefined}
                    />
                    <span
                        className="ml-2 text-gray-600 dark:text-gray-400"
                    >
                        minutes
                    </span>
                </div>
                {
                    !!actionData?.max_consumption_timeout_minutes &&
                    <p color="tomato">{actionData?.max_consumption_timeout_minutes}</p>
                }
                <Label htmlFor="ntfy_topic">NTFY topic</Label>
                <Input name="ntfy_topic" defaultValue={settings?.ntfy_topic ?? undefined}/>
                {
                    !!actionData?.ntfy_topic &&
                    <p color="tomato">{actionData?.ntfy_topic}</p>
                }
                <Button
                    className="mr-1 mt-2"
                    type="submit"
                    disabled={navigation.state === 'submitting'}
                >
                    {navigation.state === 'submitting' ? "Updating" : "Update"}
                </Button>
            </Form>

        </div>
    );
};

export default Notifications;
