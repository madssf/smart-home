import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Badge, Heading, Text} from "@chakra-ui/react";
import {getConsumption, getCurrentPrice, getRoomTemps} from "~/routes/index.server";
import {useFetcher, useLoaderData} from "@remix-run/react";
import type {Consumption, LiveConsumption, Price, RoomTemp} from "./types";
import {PriceLevel} from "./types";
import React, {useEffect, useState} from "react";
import ConsumptionGraph from "~/components/consumptionGraph";
import type {LiveConsumptionData} from "~/routes/liveData";
import {ClientOnly} from "remix-utils";

interface ResponseData {
    price: Price;
    consumption: Consumption[];
    roomTemps: RoomTemp[];
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const [price, consumption, roomTemps] = await Promise.all([
        getCurrentPrice(),
        getConsumption(),
        getRoomTemps(),
    ]);

    return json<ResponseData>({
        price,
        consumption,
        roomTemps,
    });

};

export default function Index() {

    const data = useLoaderData<ResponseData>();
    const liveFetcher = useFetcher<LiveConsumptionData>();
    const [fetchTrigger, setFetchTrigger] = useState(0);


    useEffect(() => {
        liveFetcher.load("/liveData");
        const interval = setInterval(() => {
           setFetchTrigger((prev) => prev + 1);
        }, 2500);
        return () => clearInterval(interval);

    }, [fetchTrigger]);

    const getColorScheme = (priceLevel: PriceLevel) => {
        switch (priceLevel) {
            case PriceLevel.CHEAP:
                return 'green';
            case PriceLevel.NORMAL:
                return 'blue';
            case PriceLevel.EXPENSIVE:
                return 'red';

        }
    };

    const getLiveConsumption = (data?: LiveConsumption[]): { consumption: number | null, consumptionColor: string } => {
        if (data === undefined || data.length === 0) {
            return { consumption: null, consumptionColor: 'gray' };
        } else if (data.length === 1) {
            return { consumption: data[0].power, consumptionColor: 'gray' };
        } else {
            return {
                consumption: data[0].power,
                consumptionColor: data[0].power === data[1].power ? 'gray' :
                    data[0].power > data[1].power ?
                        'red' : 'green',
            };
        }
    };

    const { consumption, consumptionColor } = getLiveConsumption(liveFetcher.data?.liveConsumption);

    return (
        <div>
            <Heading>
                Smart Home
            </Heading>
            <div className="flex flex-col">
                <div className="my-4 flex flex-col">
                    <div className="grid grid-cols-[110px_auto] p-1">
                        <b>Power usage</b>
                        <Badge maxW={"max-content"} ml={1} fontSize="md" colorScheme={consumptionColor}>{consumption ?? '-'} W</Badge>
                    </div>
                    <div className="grid grid-cols-[110px_auto] p-1">
                        <b>Current price</b>
                        <Badge maxW={"max-content"} ml={1} fontSize="md" colorScheme={getColorScheme(data.price.level)}>
                            {data.price.amount.toFixed(2)} {data.price.currency} - {data.price.level}
                        </Badge>
                    </div>
                    <div className="my-1">
                    {
                        data.roomTemps.map((roomTemp) => {
                            return <Text key={roomTemp.room_name} ml={1}><b>{roomTemp.room_name}: </b>{roomTemp.temp} °C</Text>;
                        })
                    }
                    </div>
                </div>
                <ClientOnly>
                    {
                        () => <ConsumptionGraph consumption={data.consumption} />
                    }
                </ClientOnly>
            </div>
        </div>
    );
}
