import type {NewRequest} from "~/fetcher/fetcher.server";
import {createRequest, getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {NotificationSettings} from "~/routes/notifications/types";

export async function getNotificationSettings(): Promise<NotificationSettings | null> {
    return await getRequest<NotificationSettings | null>(apiRoutes.notification_settings);
}

export async function upsertNotificationSettings(settings: NewRequest<NotificationSettings>): Promise<void> {
    return await createRequest<NewRequest<NotificationSettings>>(apiRoutes.notification_settings, settings);
}
