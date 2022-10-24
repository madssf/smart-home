import {getSessionData} from "~/utils/auth.server";
import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Link, useLoaderData} from "@remix-run/react";
import React from "react";

export interface IndexData {
    idToken?: string
}

export const handle = {hydrate: true};

export const loader: LoaderFunction = async ({request}) => {
    const {idToken} = await getSessionData(request);

    return json<IndexData>({
        idToken,
    });
};

export default function Index() {

    const data = useLoaderData<IndexData>();

    return (
        <div>
            <h1 className="text-4xl mb-5">
                Smart Home
            </h1>
            {
                data.idToken ?
                    <Link to={"/home"}>
                        Home
                    </Link>
                    :
                    <p>Please log in!</p>
            }
        </div>
    );
}

export function ErrorBoundary({error}: { error: Error }) {
    console.error(error);

    return (
        <div>
            <p>{error.message}</p>
        </div>
    );
}
