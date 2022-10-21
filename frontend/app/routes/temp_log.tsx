import React from 'react';
import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {useLoaderData, useNavigate, useSearchParams} from "@remix-run/react";
import {Line, LineChart, Tooltip, XAxis} from 'recharts';
import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import timezone from "dayjs/plugin/timezone";
import {Button, Heading} from "@chakra-ui/react";
import {ClientOnly} from "remix-utils";
import {routes} from "~/routes";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {getTemperatureLogs} from "~/routes/temp_log/temp_log.server";

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

export const loader: LoaderFunction = async ({request}) => {

    await requireUserId(request);

    const url = new URL(request.url);
    const period: TimePeriod = TimePeriod[(url.searchParams.get("period")  ?? 'day') as keyof typeof TimePeriod];

    const tempLogs = await getTemperatureLogs();

    dayjs.extend(utc);
    dayjs.extend(timezone);
    const now = dayjs().tz(process.env.TIMEZONE!);

    const generateDataset = () => {
        const temps = tempLogs
            .map((entry) => {
                return {
                    temp: entry.temp,
                    time: dayjs(entry.time),
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
    const loaderData = useLoaderData<ResponseData>();

    const [searchParams] = useSearchParams();
    const navigate = useNavigate();

    return (
        <div>
            <Heading className="pb-4">Temperature log</Heading>
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
                                onClick={() => navigate(`${routes.TEMP_LOG.ROOT}?period=${period}`)}>
                                {capitalizeAndRemoveUnderscore(period)}
                            </Button>;
                        })}
                    </div>
            </div>
            <div className="flex justify-center">
                <ClientOnly>
                    {() => <LineChart width={350} height={200} data={loaderData.dataset}>
                        <Line type="monotone" dataKey={'temp'} stroke="#8884d8" />
                        <XAxis dataKey="timeString" />
                        <Tooltip />
                    </LineChart>}
                </ClientOnly>
            </div>


        </div>
    );
};

export default TempLog;
