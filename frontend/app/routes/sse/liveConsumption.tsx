import type {LoaderFunctionArgs} from "@remix-run/node";

import {eventStream} from "remix-utils/sse/server";
import {BASE_URL} from "~/fetcher/fetcher.server";
import {apiRoutes} from "~/fetcher/apiRoutes";
import EventSource from "eventsource";

export async function loader({ request }: LoaderFunctionArgs) {
    return eventStream(request.signal, function setup(send) {
        const url = BASE_URL + apiRoutes.prices.live_consumption_sse;
        const eventSource = new EventSource(url);
        eventSource.addEventListener(
            "message",
            function (e: MessageEvent) {
                send(e);
            },
            false
        );

        return function clear() {
            eventSource.close();
        };
    });
}
