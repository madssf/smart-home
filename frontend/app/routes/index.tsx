import {json, Link, LoaderFunction, useLoaderData} from "remix";
import {getSessionData} from "~/utils/auth.server";

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
            <h1 className="text-4xl">Smart Home</h1>
            {
                data.idToken ?
                    <Link to={"/home"}>
                        Home
                    </Link>
                    :
                    <p>Log in dude!</p>
            }
        </div>
    );
}
