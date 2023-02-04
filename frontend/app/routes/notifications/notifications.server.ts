import type {CreateRequest} from "~/fetcher/fetcher.server";
import {createRequest, getRequest} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import type {NotificationSettings} from "~/routes/notifications/types";

export async function getNotificationSettings(): Promise<NotificationSettings | null> {
    return await getRequest<NotificationSettings | null>(apiRoutes.notification_settings);
}

export async function upsertNotificationSettings(settings: CreateRequest<NotificationSettings>): Promise<void> {
    return await createRequest<CreateRequest<NotificationSettings>>(apiRoutes.notification_settings, settings);
}
