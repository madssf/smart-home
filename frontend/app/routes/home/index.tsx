import {requireUserId} from "~/utils/sessions.server";
import {json, LoaderFunction} from "@remix-run/node";
import {Link, useLoaderData} from "@remix-run/react";
import {Heading} from "@chakra-ui/react";

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
            <Heading>
                Smart Home
            </Heading>
            <p>{data.name}</p>
            <div>
                <Link to={'/plugs'}>Plugs</Link>
                <Link to={'/schedules'}>Schedules</Link>
            </div>


        </div>
    );
}
