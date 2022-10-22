import React from 'react';
import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {useLoaderData, useNavigate, useParams, useSearchParams} from "@remix-run/react";
import {Line, LineChart, Tooltip, XAxis, YAxis} from 'recharts';
import dayjs from "dayjs";
import {Button} from "@chakra-ui/react";
import {ClientOnly} from "remix-utils";
import {routes} from "~/routes";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {getRoomTemperatureLogs} from "~/routes/temp_log/temp_log.server";
import {getNow} from "~/utils/time";

type DatasetEntry = {timeString: string, temp: number}

type ResponseData = {
    dataset: DatasetEntry[]
}

export type TempLogType = {
    room: string,
    temp: string,
    time: string,
}

export enum TimePeriod {
    day = 'day',
    week = 'week',
    month = 'month',
}

export const loader: LoaderFunction = async ({request, params}) => {

    await requireUserId(request);

    const room_id = params.room_id!;

    const url = new URL(request.url);
    const period: TimePeriod = TimePeriod[(url.searchParams.get("period")  ?? 'day') as keyof typeof TimePeriod];

    const tempLogs = await getRoomTemperatureLogs(room_id);

    const now = getNow();

    const generateDataset = () => {
        const temps = tempLogs
            .map((entry) => {
                return {
                    temp: entry.temp,
                    time: dayjs(entry.time).tz(process.env.TIMEZONE, true),
                };
            });

        const length = () => {
            switch (period) {
                case TimePeriod.day:
                    return 24;
                case TimePeriod.week:
                    return 7;
                case TimePeriod.month:
                    return 30;
            }
        };

        const gap = () => {
            switch (period) {
                case TimePeriod.day:
                    return 'hour';
                case TimePeriod.month:
                case TimePeriod.week:
                    return 'day';
            }
        };

        const format = () => {
            switch (period) {
                case TimePeriod.day:
                    return 'HH:mm';
                case TimePeriod.week:
                    return 'ddd';
                case TimePeriod.month:
                    return 'DD/MM';
            }
        };

        const getTempValue = (time: dayjs.Dayjs) => {
            switch (period) {
                case TimePeriod.day:
                    return Number(temps.reduce((prev, curr) => {
                        return (Math.abs(curr.time.diff(time)) < Math.abs(prev.time.diff(time)) ? curr : prev);
                    }).temp);
                case TimePeriod.week:
                case TimePeriod.month:
                    const dayTemps = temps
                        .filter((entry) =>
                            entry.time.date() === time.date() && time.month() === entry.time.month() && entry.time.year() === time.year());
                    if (dayTemps.length === 0) {
                        return null;
                    }
                    const day = [...Array(24).keys()].map((i) => {
                        const hour = time.startOf('day').add(i, 'hour');
                        return Number(dayTemps.reduce((prev, curr) => {
                            return (Math.abs(curr.time.diff(hour)) < Math.abs(prev.time.diff(hour)) ? curr : prev);
                        }).temp);
                    });
                    return day.reduce((acc, curr) => acc + curr, 0) / day.length;
            }

        };

        function notEmpty(value: DatasetEntry | { timeString: string, temp: number | null }): value is DatasetEntry {
            return value.temp !== null;
        }

        return [...Array(length()).keys()].map((i) => {
            const time = now.subtract(i, gap());
            return {
                timeString: time.format(format()),
                temp: getTempValue(time),
            };
        })
            .filter(notEmpty)
            .reverse();

    };

    if (tempLogs.length === 0) {
        return json({
            dataset: [],
        });
    }

    return json<ResponseData>({
        dataset: generateDataset().map((entry) => {return {temp: Math.round(entry.temp * 10) / 10, timeString: entry.timeString}}),
    });

};

export const handle = {hydrate: true};

const TempLog = () => {
    const {room_id} = useParams();
    const loaderData = useLoaderData<ResponseData>();

    const [searchParams] = useSearchParams();
    const navigate = useNavigate();
    const domainMin = Math.round(loaderData.dataset.reduce((a, b) => b.temp < a ? b.temp : a, Infinity)) - 3;
    const domainMax = Math.round(loaderData.dataset.reduce((a, b) => b.temp > a ? b.temp : a, 0)) + 3;


    return (
        <div className="mt-4">
            <div className="flex flex-col">
                <div className='grid grid-cols-3 px-8 pb-8'>
                    {Object.values(TimePeriod).map((period) => {
                        return <Button
                            size='md'
                            className='w'
                            key={period}
                            id="period"
                            name="period"
                            disabled={(searchParams.get('period') ?? 'day') === period}
                            value={period}
                            onClick={() => navigate(`${routes.TEMP_LOG.ROOM_ID(room_id!)}?period=${period}`)}>
                            {capitalizeAndRemoveUnderscore(period)}
                        </Button>;
                    })}
                </div>
            </div>
            <div className="flex justify-center">
                <ClientOnly>
                    {() =>
                        <LineChart margin={{bottom: 40}} width={360} height={300} data={loaderData.dataset}>
                            <Line type="monotone" dataKey={'temp'} stroke="#8884d8" strokeWidth={1.5} />
                            <XAxis padding={{right: 4}} interval={'preserveEnd'} dataKey="timeString" tick={<CustomizedAxisTick />} />
                            <YAxis type="number" padding={{bottom: 40}} mirror domain={[domainMin, domainMax]} />

                            <Tooltip />
                        </LineChart>
                            }
                </ClientOnly>
            </div>


        </div>
    );
};


function CustomizedAxisTick({ x, y, stroke, payload }: any) {

    return (
        <g transform={`translate(${x},${y})`}>
            <text x={12} y={0} dy={16} textAnchor="end" fill="#666" transform="rotate(-35)">
                {payload.value}
            </text>
        </g>
    );

}


export default TempLog;
