import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Badge, Heading, Link, Tab, TabList, TabPanel, TabPanels, Tabs, Text} from "@chakra-ui/react";
import {
    getActiveSchedules,
    getConsumption,
    getCurrentPrice,
    getPlugStatuses,
    getRoomTemps,
} from "~/routes/index.server";
import {useFetcher, useLoaderData} from "@remix-run/react";
import type {ActiveSchedule, Consumption, PlugStatus, PriceInfo, RoomTemp} from "./types";
import {PriceLevel} from "./types";
import React, {useEffect, useState} from "react";
import ConsumptionGraph from "~/components/consumptionGraph";
import type {LiveConsumptionChange, LiveConsumptionData} from "~/routes/liveData";
import {ClientOnly} from "remix-utils";
import {formatNumber, formatPriceInfo} from "~/utils/formattingUtils";
import dayjs from "dayjs";

import relativeTime from "dayjs/plugin/relativeTime";
import LiveConsumptionGraph from "~/components/liveConsumptionGraph";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";
import {routes} from "~/routes";

interface ResponseData {
    rooms: Room[],
    activeSchedules: ActiveSchedule[],
    price: PriceInfo;
    consumption: Consumption[];
    roomTemps: RoomTemp[];
    plugStatuses: PlugStatus[];
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const [rooms, activeSchedules, price, consumption, roomTemps, plugStatuses] = await Promise.all([
        getRooms(),
        getActiveSchedules(),
        getCurrentPrice(),
        getConsumption(),
        getRoomTemps(),
        getPlugStatuses(),
    ]);

    return json<ResponseData>({
        rooms,
        activeSchedules,
        price,
        consumption,
        roomTemps,
        plugStatuses,
    });

};

export default function Index() {

    const data = useLoaderData<ResponseData>();
    const liveFetcher = useFetcher<LiveConsumptionData>();
    const [fetchTrigger, setFetchTrigger] = useState(0);
    dayjs.extend(relativeTime);

    useEffect(() => {
        liveFetcher.load("/liveData");
        const interval = setInterval(() => {
           setFetchTrigger((prev) => prev + 1);
        }, 2500);
        return () => clearInterval(interval);

    }, [fetchTrigger]);

    const getColorForPrice = (priceLevel: PriceLevel) => {
        switch (priceLevel) {
            case PriceLevel.VeryCheap:
            case PriceLevel.Cheap:
                return 'green';
            case PriceLevel.Normal:
                return 'blue';
            case PriceLevel.Expensive:
            case PriceLevel.VeryExpensive:
                return 'red';

        }
    };

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

    const renderRoomData = (room: Room, roomTemp: RoomTemp | undefined, plugs: PlugStatus[], activeSchedule: ActiveSchedule | undefined) => {
        return (
            <div className="ml-1 mb-1">
                <div className="flex flex-row items-baseline">
                    <Link href={routes.TEMP_LOG.ROOM_ID(room.id)} fontWeight='bold'>{room.name}</Link>
                    {
                        roomTemp &&
                        <div className="ml-2 grid grid-cols-[65px_auto] gap-1 p-1">
                            <Badge className="text-left w-16" fontSize="md">{`${formatNumber(roomTemp.temp, 1, 1)} °C`}</Badge>
                            <p className={"ml-1"}>{dayjs(roomTemp.time).fromNow()}</p>
                        </div>
                    }
                </div>
                <div className="ml-1">
                    <div className="grid grid-cols-[70px_auto] gap-1 p-1">
                        <Text>Schedule</Text>
                        {activeSchedule?.schedule && activeSchedule.temp ?
                                <Badge colorScheme={'blue'} className="text-left w-16" fontSize="md">
                                    {`${formatNumber(activeSchedule.temp, 1, 1)} °C`}
                                </Badge>
                            : <Badge
                                maxW={"max-content"}
                                ml={1}
                                fontSize="md"
                                colorScheme='gray'
                            >
                                Off
                            </Badge>
                        }
                    </div>
                    {
                        plugs.length > 0 &&
                        <div className="ml-1">
                            <Text>Plugs</Text>
                            {
                                plugs.sort((a, b) => a.name.localeCompare(b.name)).map((plugStatus) => {
                                    return (
                                        <div key={plugStatus.name} className="grid grid-cols-[100px_auto] gap-1 p-1">
                                            <Text>{plugStatus.name}</Text>
                                            <Badge
                                                maxW={"max-content"}
                                                ml={1}
                                                fontSize="md"
                                                colorScheme={plugStatus.is_on ? 'blue' : 'gray'}
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
            <Heading>
                Smart Home
            </Heading>
            <div className="flex flex-col">
                <div className="my-4 flex flex-col">
                    <Heading size='md' mb={1}>Power</Heading>
                    <div className="my-2">
                        <Tabs maxW={"min"}>
                            <TabList>
                                <Tab>Live</Tab>
                                <Tab>Today</Tab>
                            </TabList>

                            <TabPanels>
                                <TabPanel px={0}>
                                    <div>
                                        <ClientOnly>
                                            {
                                                () => <LiveConsumptionGraph liveConsumption={consumptionGraphData} />

                                            }
                                        </ClientOnly>
                                        <div className="grid grid-cols-[110px_auto_auto] p-1">
                                            <b>Consumption</b>
                                            <div className="flex flex-row">
                                                <Badge
                                                    maxW={"max-content"}
                                                    ml={1}
                                                    fontSize="md"
                                                    colorScheme={getColorForConsumptionChange(consumptionStats?.consumptionChange)}
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
                                            <Badge
                                                maxW={"max-content"}
                                                ml={1}
                                                fontSize="md"
                                                colorScheme={getColorForPrice(data.price.price_level ?? data.price.ext_price_level)}
                                            >
                                                {formatPriceInfo(data.price)}
                                            </Badge>
                                        </div>
                                    </div>
                                </TabPanel>
                                <TabPanel px={0}>
                                    <ClientOnly>
                                        {
                                            () => {
                                                return data.consumption.length === 0 ?
                                                    <p>No consumption data</p>
                                                    :
                                                    <ConsumptionGraph consumption={data.consumption} />;
                                            }
                                        }
                                    </ClientOnly>
                                </TabPanel>
                            </TabPanels>
                        </Tabs>

                    </div>


                </div>
                <div className="my-1">
                    <Heading size='md' mb={1}>Rooms</Heading>
                    {
                        data.rooms.sort((a, b) => a.name.localeCompare(b.name)).map((room) => {
                            return renderRoomData(
                                room,
                                data.roomTemps.find(room_temp => room_temp.room_id === room.id),
                                data.plugStatuses.filter((plug) => plug.room_id === room.id),
                                data.activeSchedules.find(schedule => schedule.room_id === room.id),
                            );
                        })
                    }
                </div>
            </div>
        </div>
    );
}
