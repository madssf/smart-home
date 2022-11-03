import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Badge, Heading} from "@chakra-ui/react";
import {getConsumption, getCurrentPrice, getPlugStatuses, getRoomTemps} from "~/routes/index.server";
import {useFetcher, useLoaderData} from "@remix-run/react";
import type {Consumption, PlugStatus, Price, RoomTemp} from "./types";
import {PriceLevel} from "./types";
import React, {useEffect, useState} from "react";
import ConsumptionGraph from "~/components/consumptionGraph";
import type {LiveConsumptionChange, LiveConsumptionData} from "~/routes/liveData";
import {ClientOnly} from "remix-utils";
import {formatNumber} from "~/utils/formattingUtils";
import dayjs from "dayjs";

import relativeTime from "dayjs/plugin/relativeTime";
import LiveConsumptionGraph from "~/components/liveConsumptionGraph";

interface ResponseData {
    price: Price;
    consumption: Consumption[];
    roomTemps: RoomTemp[];
    plugStatuses: PlugStatus[];
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const [price, consumption, roomTemps, plugStatuses] = await Promise.all([
        getCurrentPrice(),
        getConsumption(),
        getRoomTemps(),
        getPlugStatuses(),
    ]);

    return json<ResponseData>({
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
            case PriceLevel.CHEAP:
                return 'green';
            case PriceLevel.NORMAL:
                return 'blue';
            case PriceLevel.EXPENSIVE:
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
                        <ClientOnly>
                            {
                                () => <LiveConsumptionGraph liveConsumption={consumptionGraphData} />

                            }
                        </ClientOnly>
                    </div>
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
                            {consumptionStats?.consumptionTime && Math.abs(dayjs(consumptionStats.consumptionTime).diff(dayjs(), 'seconds')) > 10 &&
                                <p className={"ml-1"}>{dayjs(consumptionStats.consumptionTime).fromNow()}</p>
                            }
                        </div>
                    </div>
                    <div className="grid grid-cols-[110px_auto] p-1">
                        <b>Current price</b>
                        <Badge maxW={"max-content"} ml={1} fontSize="md" colorScheme={getColorForPrice(data.price.level)}>
                            {data.price.amount.toFixed(2)} {data.price.currency} - {data.price.level}
                        </Badge>
                    </div>

                </div>
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
                <div className="my-1">
                    <Heading size='md' mb={1}>Temperatures</Heading>
                    {
                        data.roomTemps.sort((a, b) => a.room_name.localeCompare(b.room_name)).map((roomTemp) => {
                            return (
                                <div key={roomTemp.room_name} className="grid grid-cols-[130px_70px_auto] gap-1 p-1">
                                    <b>{roomTemp.room_name}</b>
                                    <Badge ml={1} className="text-left w-16" fontSize="md">{`${formatNumber(roomTemp.temp, 1, 1)} °C`}</Badge>
                                    <p className={"ml-1"}>{dayjs(roomTemp.time).fromNow()}</p>
                                </div>
                            );
                        })
                    }
                </div>
                <div className="my-1">
                    <Heading size='md' mb={1}>Plugs</Heading>
                    {
                        data.plugStatuses.sort((a, b) => a.name.localeCompare(b.name)).map((plugStatus) => {
                            return (
                                <div key={plugStatus.name} className="grid grid-cols-[130px_80px_auto] gap-1 p-1">
                                    <b>{plugStatus.name}</b>
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
            </div>
        </div>
    );
}
