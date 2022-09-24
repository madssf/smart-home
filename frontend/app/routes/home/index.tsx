import {json, LoaderFunction, redirect, useLoaderData} from "remix";
import {getSessionData} from "~/utils/auth.server";
import {db} from "~/utils/firebase.server";
import {Schedule} from "~/routes/home/types";

export interface ScheduleData {
    schedules: Schedule[];
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
        schedules: docs as Schedule[],
    });


};

export default function Index() {

    const data = useLoaderData<ScheduleData>()

    return (
        <div>
            <h1 className="text-3xl font-bold underline">
                Smart Home
            </h1>
            {
                data.schedules.map((schedule) => {
                    return (
                        <div>
                            <b>{schedule.id}</b>
                            <p>{schedule.level}</p>
                            <ul>{schedule.days.map((day) => {
                                return <li key={schedule.id + day}>{day}</li>
                            })}
                            </ul>
                            <ul>{schedule.hours.map((window) => {
                                return <li key={schedule.id + window.from + window.to}>{`From ${window.from.slice(0, 5)} to ${window.to.slice(0, 5)}`}</li>
                            })}
                            </ul>
                        </div>
                    )
                })
            }
        </div>
    );
}
