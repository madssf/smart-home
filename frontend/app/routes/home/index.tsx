import {requireUserId} from "~/utils/sessions.server";
import {json, LoaderFunction} from "@remix-run/node";
import {useLoaderData} from "@remix-run/react";
import {Link} from "@chakra-ui/react";
import {routes} from "~/routes";

interface ResponseData {
    name: string;
}


export const loader: LoaderFunction = async ({request}) => {

    const { name } = await requireUserId(request)

    return json<ResponseData>({
        name: name,
    });

};

export default function Index() {

    const data = useLoaderData<ResponseData>()

    return (
        <div>
            <h1 className="text-4xl mb-5">
                Smart Home
            </h1>
            <p>Welcome, <b>{data.name}</b></p>
            <div className="flex flex-col ml-4">
                <Link className="mt-2" href={routes.PLUGS.ROOT}>Plugs</Link>
                <Link className="mt-2" href={routes.SCHEDULES.ROOT}>Schedules</Link>
                <Link className="mt-2" href={routes.TEMP_ACTIONS.ROOT}>Temp actions</Link>
                <Link className="mt-2" href={routes.TEMP_LOG.ROOT}>Temperature log</Link>
            </div>

        </div>
    );
}
