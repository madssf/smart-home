import {requireUserId} from "~/utils/sessions.server";
import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {Heading, Link} from "@chakra-ui/react";
import {routes} from "~/routes";

interface ResponseData {
    name: string;
}


export const loader: LoaderFunction = async ({request}) => {

    const { name } = await requireUserId(request);

    return json<ResponseData>({
        name: name,
    });

};

export default function Index() {


    return (
        <div>
            <Heading className="flex justify-center">
                Smart Home
            </Heading>
            <div className="flex flex-col ml-4">
                <Link className="mt-2" href={routes.PLUGS.ROOT}>Plugs</Link>
                <Link className="mt-2" href={routes.SCHEDULES.ROOT}>Schedules</Link>
                <Link className="mt-2" href={routes.TEMP_ACTIONS.ROOT}>Temp actions</Link>
                <Link className="mt-2" href={routes.TEMP_LOG.ROOT}>Temperature log</Link>
            </div>

        </div>
    );
}
