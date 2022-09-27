import {getSessionData} from "~/utils/auth.server";
import {json, LoaderFunction} from "@remix-run/node";
import {Link, useLoaderData} from "@remix-run/react";
import React from "react";

export interface IndexData {
    csrf?: string;
    idToken?: string
}

export const loader: LoaderFunction = async ({request}) => {
    const {csrf, idToken} = await getSessionData(request);

    return json<IndexData>({
        csrf,
        idToken,
    });
};

export default function Index() {

    const data = useLoaderData<IndexData>()

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
