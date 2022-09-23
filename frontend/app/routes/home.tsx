import {json, LoaderFunction, redirect, useLoaderData} from "remix";
import {getSessionData} from "~/utils/auth.server";
import {db} from "~/utils/firebase.server";

export interface ScheduleData {
    schedules: string;
}


export const loader: LoaderFunction = async ({request}) => {
    const {idToken} = await getSessionData(request);

    if (!idToken) {
        throw redirect("/")
    }

    const schedulesRef = await db.collection('schedules')
    const schedules = await schedulesRef.get()
    const docs = schedules.docs.map((a) => {
        return a.data()
    })
    return json<ScheduleData>({
        schedules: JSON.stringify(docs),
    });


};

export default function Index() {

    const data = useLoaderData<ScheduleData>()

    return (
        <div>
            <h1>Smart Home</h1>
            {
                data.schedules
            }
        </div>
    );
}
