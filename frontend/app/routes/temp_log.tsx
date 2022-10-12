import React from 'react';
import {json, LoaderFunction} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {db} from "~/utils/firebase.server";
import {collections} from "~/utils/firestoreUtils.server";
import {useLoaderData} from "@remix-run/react";

type ResponseData = {
    tempLogs: TempLog[]
}

export type TempLog = {
    room: string,
    temp: number,
    time: string,
}

export const loader: LoaderFunction = async ({request}) => {

    const {userId} = await requireUserId(request)

    const tempLogRef = await db.collection(collections.tempLog(userId)).get()
    const tempLogs = tempLogRef.docs.map((doc) => {
        const data = doc.data()
        const log: TempLog = {
            room: data.room, temp: data.temp, time: data.time
        }
        return log
    })
    return json<ResponseData>({
        tempLogs,
    });

};

const TempLog = () => {
    const loaderData = useLoaderData<ResponseData>()

    return (
        <div>
            {
                 loaderData.tempLogs
                     .sort((a, b) => a.time.localeCompare(b.time))
                     .map((log) => {
                         return <p>{log.time.slice(0, 16)} {log.temp}</p>
                 })
            }
        </div>
    );
};

export default TempLog;
