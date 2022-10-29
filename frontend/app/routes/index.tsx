import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Heading} from "@chakra-ui/react";
import {getCurrentPrice} from "~/routes/index.server";
import {useLoaderData} from "@remix-run/react";
import type {Price} from "./types";
import {pageLinks} from "~/components/pageLinks";
import React from "react";

interface ResponseData {
    price: Price;
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async () => {

    const price = await getCurrentPrice();

    return json<ResponseData>({
        price,
    });

};

export default function Index() {

    const data = useLoaderData<ResponseData>();

    return (
        <div className="ml-4">
            <Heading className="flex justify-center">
                Smart Home
            </Heading>
            <div className="mt-4">
                <p><b>Current price: </b>{data.price.amount.toFixed(2)} {data.price.currency} - {data.price.level}</p>
            </div>
            <div className="flex flex-col mt-4">
                {pageLinks}
            </div>
        </div>
    );
}
