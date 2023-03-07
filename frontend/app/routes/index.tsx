import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {
    Alert,
    AlertIcon,
    Badge,
    FormControl,
    FormLabel,
    Heading,
    Link,
    Switch,
    Tab,
    TabList,
    TabPanel,
    TabPanels,
    Tabs,
    Text,
} from "@chakra-ui/react";
import {
    enrichRoomData,
    getActiveSchedules,
    getConsumptionOrError,
    getCurrentPriceOrError,
    getPlugStatuses,
    getRoomTemps,
} from "~/routes/index.server";
import {useFetcher, useLoaderData} from "@remix-run/react";
import type {Consumption, EnrichedRoomData, PriceInfo} from "./types";
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
import {routes} from "~/routes";
import type {DataOrError} from "~/fetcher/fetcher.server";

interface ResponseData {
    rooms: EnrichedRoomData[],
    price: DataOrError<PriceInfo>;
    consumption: DataOrError<Consumption[]>;
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

    const getColorForPrice = (priceLevel: PriceLevel) => {
        switch (priceLevel) {
            case PriceLevel.VeryCheap:
                return 'green';
            case PriceLevel.Cheap:
                return 'cyan';
            case PriceLevel.Normal:
                return 'blue';
            case PriceLevel.Expensive:
                return 'orange';
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

    const roomsToRender = (rooms: EnrichedRoomData[]) => {
        if (hideUnscheduledRooms) {
            return rooms.filter(r => r.plugStatuses.some(p => p.scheduled));
        }
        return rooms;
    };

    const renderRooms = (rooms: EnrichedRoomData[]) => {
        if (rooms.length === 0) {
            const message = hideUnscheduledRooms ? 'No rooms with schedule' : 'No rooms added yet';
            return <Alert mt={2} status="info">
                <AlertIcon />
                {message}
            </Alert> ;
        } else {
            return rooms.sort((a, b) => a.name.localeCompare(b.name)).map((room) => {
                return <React.Fragment key={room.id}>
                    { renderRoomData(room) }
                </React.Fragment>;
            });
        }
    };

    const renderRoomData = (room: EnrichedRoomData) => {
        return (
            <div className="ml-1 mb-1">
                <div className="flex flex-row items-baseline">
                    <Link href={routes.TEMP_LOG.ROOM_ID(room.id)} fontWeight='bold'>{room.name}</Link>
                    {
                        room.temp &&
                        <div className="ml-2 grid grid-cols-[65px_auto] gap-1 p-1">
                            <Badge className="text-left w-16"
                                   fontSize="md">{`${formatNumber(room.temp.temp, 1, 1)} °C`}
                            </Badge>
                            <p className={"ml-1"}>{dayjs(room.temp.time).fromNow()}</p>
                        </div>
                    }
                </div>
                <div className="ml-1">
                    <div className="grid grid-cols-[70px_auto] gap-1 p-1">
                        <Text>Schedule</Text>
                        {room.activeSchedule?.schedule && room.activeSchedule.temp ?
                            <Badge colorScheme={'blue'} className="text-left w-16" fontSize="md">
                                {`${formatNumber(room.activeSchedule.temp, 1, 1)} °C`}
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
                        room.plugStatuses.length > 0 &&
                        <div className="ml-1">
                            <Text>Plugs</Text>
                            {
                                room.plugStatuses.sort((a, b) => a.name.localeCompare(b.name)).map((plugStatus) => {
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
            <div className="flex flex-col mt-2">
                <div className="flex flex-col">
                    <Heading size='md' mb={1}>Power</Heading>
                    <div>
                        <Tabs>
                            <TabList>
                                <Tab>Live</Tab>
                                <Tab>Today</Tab>
                            </TabList>

                            <TabPanels>
                                <TabPanel px={0}>
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
                                            {
                                                data.price === 'ERROR' ?
                                                    <Badge
                                                        maxW={"max-content"}
                                                        ml={1}
                                                        fontSize="md"
                                                        colorScheme={'yellow'}
                                                    >
                                                        Unavailable
                                                    </Badge>
                                                    :
                                                    <Badge
                                                        maxW={"max-content"}
                                                        ml={1}
                                                        fontSize="md"
                                                        colorScheme={getColorForPrice(data.price.price_level ?? data.price.ext_price_level)}
                                                    >
                                                        {formatPriceInfo(data.price)}
                                                    </Badge>
                                            }
                                        </div>
                                    </div>
                                </TabPanel>
                                <TabPanel px={0}>
                                    <ClientOnly>
                                        {
                                            () => {
                                                return data.consumption === 'ERROR' ?
                                                    <Alert status="error">
                                                        <AlertIcon />
                                                        Consumption data unavailable
                                                    </Alert>
                                                    :
                                                    data.consumption.length === 0 ?
                                                        <Alert status="warning">
                                                            <AlertIcon />
                                                            No consumption data
                                                        </Alert>
                                                        :
                                                        <ConsumptionGraph consumption={data.consumption}/>;
                                            }
                                        }
                                    </ClientOnly>
                                </TabPanel>
                            </TabPanels>
                        </Tabs>

                    </div>


                </div>
                <div>
                    <div className="flex flex-row">
                        <Heading size='md' mb={1}>Rooms</Heading>
                        <FormControl ml={2} display='flex' alignItems='center'>
                            <FormLabel htmlFor='hide-unscheduled-rooms' mb='0' fontSize='xs'>
                                Hide unscheduled
                            </FormLabel>
                            <Switch
                                id='hide-unscheduled-rooms'
                                onChange={() => setHideUnscheduledRooms((prev) => !prev)}
                                defaultChecked={hideUnscheduledRooms}
                                size={'sm'}
                            />
                        </FormControl>
                    </div>

                    {
                        renderRooms(roomsToRender(data.rooms))
                    }
                </div>
            </div>
        </div>
    );
}
