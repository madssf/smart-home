import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {
    enrichRoomData,
    getActiveSchedules,
    getConsumptionOrError,
    getCurrentPriceOrError,
    getLiveConsumption,
    getPlugStatuses,
    getRoomTemps,
} from "~/routes/index.server";
import {Links, Meta, Scripts, useLoaderData, useRouteError} from "@remix-run/react";
import type {Consumption, EnrichedRoomData, PriceInfo} from "./types";
import {PriceLevel} from "./types";
import {useState} from "react";
import ConsumptionGraph from "~/components/consumptionGraph";
import type {LiveConsumptionChange, LiveConsumptionData} from "~/routes/liveData";
import {fromSseEvent} from "~/routes/liveData";
import {ClientOnly} from "remix-utils/client-only";
import {formatPriceInfo} from "~/utils/formattingUtils";
import dayjs from "dayjs";

import relativeTime from "dayjs/plugin/relativeTime.js";
import LiveConsumptionGraph from "~/components/liveConsumptionGraph";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {SimpleResult} from "~/fetcher/fetcher.server";
import {Alert, AlertDescription} from "~/components/ui/alert";
import {Badge} from "~/components/ui/badge";
import {Tabs, TabsContent, TabsList, TabsTrigger} from "~/components/ui/tabs";
import {Switch} from "~/components/ui/switch";
import {Theme, useTheme} from "remix-themes";
import {getErrorComponent} from "~/components/error";
import {useEventSource} from "remix-utils/sse/react";
import FrontPageRooms from "~/components/frontPageRooms";

interface ResponseData {
    rooms: EnrichedRoomData[],
    price: SimpleResult<PriceInfo>;
    consumption: SimpleResult<Consumption[]>;
    liveConsumption: LiveConsumptionData;
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const [rooms, activeSchedules, price, consumption, roomTemps, plugStatuses, liveConsumption] = await Promise.all([
        getRooms(),
        getActiveSchedules(),
        getCurrentPriceOrError(),
        getConsumptionOrError(),
        getRoomTemps(),
        getPlugStatuses(),
        getLiveConsumption(),
    ]);

    return json<ResponseData>({
        rooms: rooms.map((r) => enrichRoomData(r, activeSchedules, roomTemps, plugStatuses)),
        price,
        consumption,
        liveConsumption,
    });

};

export default function Index() {

    const data = useLoaderData<ResponseData>();
    const [hideUnscheduledRooms, setHideUnscheduledRooms] = useState(true);
    dayjs.extend(relativeTime);

    const liveConsumptionSseData = useEventSource("sse/liveConsumption")
    const liveConsumptionData = (liveConsumptionSseData ?
        fromSseEvent(liveConsumptionSseData)
        : null)
        ?? data.liveConsumption;

    const roomsToRender = (rooms: EnrichedRoomData[]) => {
        if (hideUnscheduledRooms) {
            return rooms.filter(r => r.plugStatuses.some(p => p.scheduled));
        }
        return rooms;
    };

    const consumptionGraphData = liveConsumptionData?.liveConsumption;
    const consumptionStats = liveConsumptionData?.liveConsumptionStats;

    return (
        <div>
            <div className="flex flex-col mt-2">
                <div className="flex flex-col">
                    <h2 className="mb-1">Power usage</h2>
                    <div>
                        <Tabs defaultValue="live-graph" className="mt-2">
                            <TabsList>
                                <TabsTrigger value="live-graph">Live</TabsTrigger>
                                <TabsTrigger value="daily-graph">Today</TabsTrigger>
                            </TabsList>
                            <TabsContent value="live-graph">
                                <div
                                    className="flex flex-col gap-1"
                                >
                                    <ClientOnly>
                                        {
                                            () => <LiveConsumptionGraph liveConsumption={consumptionGraphData}/>

                                        }
                                    </ClientOnly>
                                    <div className="grid grid-cols-[110px_auto_auto] p-1">
                                        <b>Consumption</b>
                                        <div className="flex flex-row gap-1 items-end">
                                            <Badge
                                                className="w-max text-left"
                                                variant={getVariantForConsumptionChange(consumptionStats?.consumptionChange)}
                                            >
                                                {consumptionStats?.consumption ?? '-'} W
                                            </Badge>
                                            {consumptionStats?.consumptionTime &&
                                                Math.abs(dayjs(consumptionStats.consumptionTime).diff(dayjs(), 'seconds')) > 10 &&
                                                <p className="ml-2 mb-0 text-sm">{dayjs(consumptionStats.consumptionTime).fromNow()}</p>
                                            }
                                        </div>
                                    </div>
                                    <div className="grid grid-cols-[110px_auto] p-1">
                                        <b>Current price</b>
                                        {
                                            data.price === 'ERROR' ?
                                                <Badge
                                                    className="max-w-max ml-1"
                                                    variant="destructive"
                                                >
                                                    Unavailable
                                                </Badge>
                                                :
                                                <Badge
                                                    className="max-w-max ml-1"
                                                    variant={getVariantForPriceInfo(data.price)}
                                                >
                                                    {formatPriceInfo(data.price)}
                                                </Badge>
                                        }
                                    </div>
                                </div>
                            </TabsContent>
                            <TabsContent value="daily-graph">
                                <ClientOnly>
                                    {
                                        () => {
                                            return data.consumption === 'ERROR' ?
                                                <Alert variant="destructive">
                                                    <AlertDescription>
                                                        Consumption data unavailable
                                                    </AlertDescription>
                                                </Alert>
                                                :
                                                data.consumption.length === 0 ?
                                                    <Alert>
                                                        <AlertDescription>
                                                            No consumption data
                                                        </AlertDescription>
                                                    </Alert>
                                                    :
                                                    <ConsumptionGraph consumption={data.consumption}/>;
                                        }
                                    }
                                </ClientOnly>
                            </TabsContent>
                        </Tabs>

                    </div>


                </div>
                <div className="mt-4">
                    <div className="flex flex-row items-center gap-4">
                        <h2 className="mb-1">Rooms</h2>
                        <div className="flex items-center gap-2">
                            <label
                                className="text-xs mb-0"
                                htmlFor='hide-unscheduled-rooms'>
                                Hide unscheduled
                            </label>
                            <Switch
                                id='hide-unscheduled-rooms'
                                onCheckedChange={() => setHideUnscheduledRooms((prev) => !prev)}
                                checked={hideUnscheduledRooms}
                            />
                        </div>
                    </div>
                    <FrontPageRooms rooms={roomsToRender(data.rooms)} hideUnscheduledRooms={hideUnscheduledRooms}/>
                </div>
            </div>
        </div>
    );
}

const getVariantForConsumptionChange = (change?: LiveConsumptionChange): 'positive' | 'negative' | 'outline' => {
    switch (change) {
        case "UP":
            return 'negative';
        case "DOWN":
            return 'positive';
        case "NONE":
        case undefined:
            return 'outline';

    }
};


const getVariantForPriceInfo = (priceInfo: PriceInfo): 'positive' | 'veryPositive' | 'veryNegative' | 'negative' | 'default' => {
    if (priceInfo.price_level) {
        switch (priceInfo.price_level) {
            case PriceLevel.VeryCheap:
                return 'veryPositive';
            case PriceLevel.Cheap:
                return 'positive';
            case PriceLevel.Expensive:
                return 'negative';
            case PriceLevel.VeryExpensive:
                return 'veryNegative';
            default:
                return 'default';
        }
    } else {
        return 'default';
    }
}

export function ErrorBoundary() {
    const error = useRouteError();
    const [theme] = useTheme()

    return (
        <html
            lang="en"
        >
        <head>
            <title>Oops!</title>
            <Meta />
            <Links />
        </head>
        <body
            className={theme === Theme.DARK ? 'dark' : ''}
        >
        {getErrorComponent(error)}
        <Scripts />
        </body>
        </html>
    );
}
