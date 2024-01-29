import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {
    enrichRoomData,
    getActiveSchedules,
    getConsumptionOrError,
    getCurrentPriceOrError,
    getPlugStatuses,
    getRoomTemps,
} from "~/routes/index.server";
import {Link, Links, Meta, Scripts, useFetcher, useLoaderData, useRouteError} from "@remix-run/react";
import type {Consumption, EnrichedRoomData, PriceInfo} from "./types";
import React, {useEffect, useState} from "react";
import ConsumptionGraph from "~/components/consumptionGraph";
import type {LiveConsumptionChange, LiveConsumptionData} from "~/routes/liveData";
import {ClientOnly} from "remix-utils/client-only";
import {formatNumber, formatPriceInfo} from "~/utils/formattingUtils";
import dayjs from "dayjs";

import relativeTime from "dayjs/plugin/relativeTime.js";
import LiveConsumptionGraph from "~/components/liveConsumptionGraph";
import {getRooms} from "~/routes/rooms/rooms.server";
import {routes} from "~/routes";
import type {SimpleResult} from "~/fetcher/fetcher.server";
import {Alert, AlertDescription} from "~/components/ui/alert";
import {Badge} from "~/components/ui/badge";
import {Tabs, TabsContent, TabsList, TabsTrigger} from "~/components/ui/tabs";
import {Switch} from "~/components/ui/switch";
import {Theme, useTheme} from "remix-themes";
import {getErrorComponent} from "~/components/error";

interface ResponseData {
    rooms: EnrichedRoomData[],
    price: SimpleResult<PriceInfo>;
    consumption: SimpleResult<Consumption[]>;
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const [rooms, activeSchedules, price, consumption, roomTemps, plugStatuses] = await Promise.all([
        getRooms(),
        getActiveSchedules(),
        getCurrentPriceOrError(),
        getConsumptionOrError(),
        getRoomTemps(),
        getPlugStatuses(),
    ]);

    return json<ResponseData>({
        rooms: rooms.map((r) => enrichRoomData(r, activeSchedules, roomTemps, plugStatuses)),
        price,
        consumption,
    });

};

export default function Index() {

    const data = useLoaderData<ResponseData>();
    const liveFetcher = useFetcher<LiveConsumptionData>();
    const [fetchTrigger, setFetchTrigger] = useState(0);
    const [hideUnscheduledRooms, setHideUnscheduledRooms] = useState(true);
    dayjs.extend(relativeTime);

    useEffect(() => {
        liveFetcher.load("/liveData");
        const interval = setInterval(() => {
            setFetchTrigger((prev) => prev + 1);
        }, 2500);
        return () => clearInterval(interval);

    }, [fetchTrigger]);
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const getColorForConsumptionChange = (change?: LiveConsumptionChange) => {
        switch (change) {
            case "UP":
                return 'red';
            case "DOWN":
                return 'green';
            case "NONE":
            case undefined:
                return 'gray';

        }
    };

    const roomsToRender = (rooms: EnrichedRoomData[]) => {
        if (hideUnscheduledRooms) {
            return rooms.filter(r => r.plugStatuses.some(p => p.scheduled));
        }
        return rooms;
    };

    const renderRooms = (rooms: EnrichedRoomData[]) => {
        if (rooms.length === 0) {
            const message = hideUnscheduledRooms ? 'No rooms with schedule' : 'No rooms added yet';
            return <Alert className="mt-2">
                <AlertDescription>
                    {message}
                </AlertDescription>
            </Alert>;
        } else {
            return rooms.sort((a, b) => a.name.localeCompare(b.name)).map((room) => {
                return <React.Fragment key={room.id}>
                    {renderRoomData(room)}
                </React.Fragment>;
            });
        }
    };

    const renderRoomData = (room: EnrichedRoomData) => {
        return (
            <div className="ml-1 mb-1">
                <div className="flex flex-row items-baseline">
                    <Link to={routes.TEMP_LOG.ROOM_ID(room.id)}>{room.name}</Link>
                    {
                        room.temp &&
                        <div className="ml-2 grid grid-cols-[65px_auto] gap-1 p-1">
                            <Badge className="text-left w-16 text-md">
                                {`${formatNumber(room.temp.temp, 1, 1)} °C`}
                            </Badge>
                            <p className={"ml-1"}>{dayjs(room.temp.time).fromNow()}</p>
                        </div>
                    }
                </div>
                <div className="ml-1">
                    <div className="grid grid-cols-[70px_auto] gap-1 p-1">
                        <p>Schedule</p>
                        {room.activeSchedule?.schedule && room.activeSchedule.temp ?
                            <Badge variant="secondary" className="text-left w-16">
                                {`${formatNumber(room.activeSchedule.temp, 1, 1)} °C`}
                            </Badge>
                            : <Badge
                                className="max-w-max ml-1"
                                variant="outline"
                            >
                                Off
                            </Badge>
                        }
                    </div>
                    {
                        room.plugStatuses.length > 0 &&
                        <div className="ml-1">
                            <p>Plugs</p>
                            {
                                room.plugStatuses.sort((a, b) => a.name.localeCompare(b.name)).map((plugStatus) => {
                                    return (
                                        <div key={plugStatus.name} className="grid grid-cols-[100px_auto] gap-1 p-1">
                                            <p>{plugStatus.name}</p>
                                            <Badge
                                                className="max-w-max ml-1"
                                                variant={plugStatus.is_on ? 'default' : 'secondary'}
                                            >
                                                {plugStatus.is_on ? `${formatNumber(plugStatus.power, 1, 1)} W` : 'OFF'}
                                            </Badge>
                                        </div>
                                    );
                                })
                            }
                        </div>
                    }

                </div>
            </div>
        );
    };

    const consumptionGraphData = liveFetcher.data?.liveConsumption;
    const consumptionStats = liveFetcher.data?.liveConsumptionStats;

    return (
        <div>
            <h1>
                Smart Home
            </h1>
            <div className="flex flex-col mt-2">
                <div className="flex flex-col">
                    <h2 className="mb-1">Power</h2>
                    <div>
                        <Tabs>
                            <TabsList>
                                <TabsTrigger value="live-graph">Live</TabsTrigger>
                                <TabsTrigger value="daily-graph">Today</TabsTrigger>
                            </TabsList>
                            <TabsContent value="live-graph">
                                <div>
                                    <ClientOnly>
                                        {
                                            () => <LiveConsumptionGraph liveConsumption={consumptionGraphData}/>

                                        }
                                    </ClientOnly>
                                    <div className="grid grid-cols-[110px_auto_auto] p-1">
                                        <b>Consumption</b>
                                        <div className="flex flex-row">
                                            <Badge
                                                className="max-w-max ml-1"
                                                // TODO: Add color
                                            >
                                                {consumptionStats?.consumption ?? '-'} W
                                            </Badge>
                                            {consumptionStats?.consumptionTime &&
                                                Math.abs(dayjs(consumptionStats.consumptionTime).diff(dayjs(), 'seconds')) > 10 &&
                                                <p className={"ml-1"}>{dayjs(consumptionStats.consumptionTime).fromNow()}</p>
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
                                                    // TODO: Add color
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
                <div>
                    <div className="flex flex-row">
                        <h2 className="mb-1">Rooms</h2>
                        <div className="ml-2 flex align-center">
                            <label
                                className="text-xs mb-0"
                                htmlFor='hide-unscheduled-rooms'>
                                Hide unscheduled
                            </label>
                            <Switch
                                id='hide-unscheduled-rooms'
                                onChange={() => setHideUnscheduledRooms((prev) => !prev)}
                                defaultChecked={hideUnscheduledRooms}
                            />
                        </div>
                    </div>

                    {
                        renderRooms(roomsToRender(data.rooms))
                    }
                </div>
            </div>
        </div>
    );
}

export function ErrorBoundary() {
    const error = useRouteError();
    const [theme] = useTheme()

    return (
        <html>
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
