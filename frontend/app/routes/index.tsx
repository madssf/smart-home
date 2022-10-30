import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Heading} from "@chakra-ui/react";
import {getConsumption, getCurrentPrice} from "~/routes/index.server";
import {useFetcher, useLoaderData} from "@remix-run/react";
import type {Consumption, Price} from "./types";
import React, {useEffect, useState} from "react";
import ConsumptionGraph from "~/components/consumptionGraph";
import type {LiveConsumptionData} from "~/routes/liveData";
import {ClientOnly} from "remix-utils";

interface ResponseData {
    price: Price;
    consumption: Consumption[];
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const price = await getCurrentPrice();
    const consumption = await getConsumption();

    return json<ResponseData>({
        price,
        consumption,
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

    return (
        <div className="ml-4">
            <Heading>
                Smart Home
            </Heading>
            <div className="mt-4 flex flex-col">
                <p className="py-2"><b>Current power: </b>{liveFetcher.data?.liveConsumption.power ?? '-'} W</p>
                <p className="py-2"><b>Current price: </b>{data.price.amount.toFixed(2)} {data.price.currency} - {data.price.level}</p>
                <ClientOnly>
                    {
                        () => <ConsumptionGraph consumption={data.consumption} />
                    }
                </ClientOnly>
            </div>
        </div>
    );
}
