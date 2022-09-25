import {ActionArgs} from "@remix-run/server-runtime";
import {json, Link, LoaderFunction, Outlet, redirect, useActionData, useLoaderData, useLocation} from "remix";
import {NaiveTime, PRICE_LEVELS, PriceLevel, Schedule, TimeWindow, Weekday, WEEKDAYS} from "~/routes/home/types";
import {requireUserId} from "~/utils/sessions.server";
import {db} from "~/utils/firebase.server";
import ScheduleForm from "~/routes/schedules/scheduleForm";
import {routes} from "~/routes";
import {collections} from "~/utils/firestoreUtils.server";
import {validateDays, validateHours, validatePriceLevel} from "~/routes/schedules/utils";

interface ResponseData {
    schedules: Schedule[];
}

export type ScheduleFormErrors = {
    [k in keyof Schedule]?: string;
} & {
    other?: string
};


export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

    const {userId} = await requireUserId(request)

    const body = await request.formData();
    const id = body.get("id")?.toString();
    const priceLevel = body.get("priceLevel")?.toString();
    const days = body.getAll("days").map((day) => day.toString());
    const from = body.getAll("from").map((naiveTime) => naiveTime.toString());
    const to = body.getAll("to").map((naiveTime) => naiveTime.toString());


    const validated = {
        priceLevel: validatePriceLevel(priceLevel),
        days: validateDays(days),
        hours: validateHours(from, to),
    }

    if (!validated.days.valid || !validated.hours.valid || !validated.priceLevel.valid) {
        return json<ScheduleFormErrors>(
            {
                id,
                days: !validated.days.valid ? validated.days.error : undefined,
                hours: !validated.hours.valid ? validated.hours.error : undefined,
                priceLevel: !validated.priceLevel.valid ? validated.priceLevel.error : undefined,
            }
        )
    }

    const document: Omit<Schedule, 'id'> = {
        days: validated.days.data, hours: validated.hours.data, priceLevel: validated.priceLevel.data,
    }

    if (!id) {
        await db.collection(collections.schedules(userId)).add(document).catch((e) => {throw Error("Something went wrong")})
    } else {
        await db.doc(`${collections.schedules(userId)}/${id}`).set(document).catch((e) => {throw Error("Something went wrong")})
    }

    return redirect(routes.SCHEDULES.ROOT);
}

export const loader: LoaderFunction = async ({request}) => {

    const {userId} = await requireUserId(request)

    const schedulesRef = await db.collection(collections.schedules(userId)).get()
    const schedules = schedulesRef.docs.map((doc) => {
        const data = doc.data()
        // TODO: Validate
        const schedule: Schedule = {
            days: data.days, hours: data.hours, id: doc.id, priceLevel: data.level
        }
        return schedule
    })
    return json<ResponseData>({
        schedules,
    });

};

const Schedules = () => {

    const location = useLocation()
    const loaderData = useLoaderData<ResponseData>()

    const renderSchedules = (schedules: Schedule[]) => {
        return schedules.map((schedule) => {
            return (
                <ScheduleForm key={schedule.id} schedule={schedule}/>
            )
        })
    }

    return (
        <div>
            {renderSchedules(loaderData.schedules)}
            {
                location.pathname !== routes.SCHEDULES.NEW ?
                    <Link to={routes.SCHEDULES.NEW}>Add schedule</Link>
                    :
                    <Link to={routes.SCHEDULES.ROOT}>Cancel</Link>
            }
            <Outlet />
        </div>
    );
};

export default Schedules;