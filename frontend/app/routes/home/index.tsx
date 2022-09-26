import {json, Link, LoaderFunction, useLoaderData} from "remix";
import {requireUserId} from "~/utils/sessions.server";

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
            <h1 className="text-3xl font-bold underline">
                Smart Home
            </h1>
            <p>{data.name}</p>
            <div>
                <Link to={'/plugs'}>Plugs</Link>
                <Link to={'/schedules'}>Schedules</Link>
            </div>


        </div>
    );
}
