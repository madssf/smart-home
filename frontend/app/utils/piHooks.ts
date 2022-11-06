import {BASE_URL} from "~/fetcher/fetcher.server";

export const piTriggerRefresh = async() => {
    return fetch(`${BASE_URL}trigger_refresh`)
        .then(res => {
            if (!res.ok) {
                console.warn('Failed to trigger refresh, response status:', res.status);
            }
        })
        .catch((e) =>
            console.warn('Error when triggering raspi refresh:', e),
        );
};
