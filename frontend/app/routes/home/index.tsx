import {requireUserId} from "~/utils/sessions.server";
import {json, LoaderFunction} from "@remix-run/node";
import {useLoaderData} from "@remix-run/react";
import {Link} from "@chakra-ui/react";

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
                <Link href={'/plugs'}>Plugs</Link>
                <Link href={'/schedules'}>Schedules</Link>
            </div>


        </div>
    );
}
