import {ActionArgs, json, LoaderFunction, redirect} from "@remix-run/node";
import {Schedule} from "~/routes/schedules/types/types";
import {requireUserId} from "~/utils/sessions.server";
import {db} from "~/utils/firebase.server";
import ScheduleForm from "~/routes/schedules/components/scheduleForm";
import {routes} from "~/routes";
import {collections} from "~/utils/firestoreUtils.server";
import {validateDays, validateHours, validatePriceLevel} from "~/routes/schedules/utils/utils";
import {FormErrors} from "~/utils/types";
import {useLoaderData} from "@remix-run/react";
import React, {useState} from "react";
import {Button} from "@chakra-ui/react";

interface ResponseData {
    schedules: Schedule[];
}
export type ScheduleFormErrors = FormErrors<Schedule>

export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

    const {userId} = await requireUserId(request)

    const body = await request.formData();
    const id = body.get("id")?.toString();
    const priceLevel = body.get("priceLevel")?.toString();
    const days = body.getAll("days").map((day) => day.toString());
    const from = body.getAll("from").map((naiveTime) => naiveTime.toString());
    const to = body.getAll("to").map((naiveTime) => naiveTime.toString());
    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await db.doc(`${collections.schedules(userId)}/${id}`).delete().catch((e) => {throw Error("Something went wrong")})
        return redirect(routes.SCHEDULES.ROOT)
    }

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

    const loaderData = useLoaderData<ResponseData>()
    const [showNew, setShowNew] = useState(false)

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
            <Button onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add schedule'}</Button>
            {
                showNew &&
                <ScheduleForm />
            }
        </div>
    );
};

export default Schedules;