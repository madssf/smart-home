import React from 'react';
import type {LoaderFunction} from "@remix-run/node";
import {json} from "@remix-run/node";
import {useLoaderData, useNavigate, useParams, useSearchParams} from "@remix-run/react";
import {Area, AreaChart, Tooltip, XAxis, YAxis} from 'recharts';
import {Button, useColorMode} from "@chakra-ui/react";
import {ClientOnly} from "remix-utils";
import {routes} from "~/routes";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {getRoomTemperatureLogs} from "~/routes/temp_log/temp_log.server";

type DatasetEntry = {label: string, temp: number}

type ResponseData = {
    dataset: DatasetEntry[]
}

export type TempLogType = {
    label: string,
    temp: number,
}

export enum TimePeriod {
    day = 'day',
    week = 'week',
    month = 'month',
}

export const loader: LoaderFunction = async ({request, params}) => {

    const room_id = params.room_id!;

    const url = new URL(request.url);
    const period: TimePeriod = TimePeriod[(url.searchParams.get("period")  ?? 'day') as keyof typeof TimePeriod];

    const tempLogs = await getRoomTemperatureLogs(room_id, period);

    return json<ResponseData>({dataset: tempLogs});

};

export const handle = {hydrate: true};

const TempLog = () => {
    const {room_id} = useParams();
    const loaderData = useLoaderData<ResponseData>();

    const [searchParams] = useSearchParams();
    const navigate = useNavigate();
    const domainMin = Math.round(loaderData.dataset.reduce((a, b) => b.temp < a ? b.temp : a, Infinity)) - 3;
    const domainMax = Math.round(loaderData.dataset.reduce((a, b) => b.temp > a ? b.temp : a, 0)) + 3;

    const {colorMode} = useColorMode();

    const color = colorMode === 'dark' ? '#F7FAFC' : '#4A5568';

    function CustomizedAxisTick({ x, y, stroke, payload }: any) {

        return (
            <g transform={`translate(${x},${y})`}>
                <text fontSize="small" x={12} y={0} dy={16} textAnchor="end" fill={color} transform="rotate(-35)">
                    {payload.value}
                </text>
            </g>
        );

    }

    function CustomTooltip({ active, payload, label }: any) {
        if (active && payload && payload.length) {
            return (
                <div className="custom-tooltip">
                    <p className="label">{`${label} : ${payload[0].value} ??C`}</p>
                </div>
            );
        }

        return null;
    }


    return (
        <div className="mt-4">
            <div className="flex flex-col">
                <div className='grid grid-cols-3 px-8 pb-8'>
                    {Object.values(TimePeriod).map((period) => {
                        return <Button
                            size='sm'
                            variant="outline"
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
                    {
                        () =>
                        {
                            if (loaderData.dataset.length === 0) {
                                return <p> No temperature data here..!</p>;
                            } else {
                                return (
                                    <AreaChart margin={{bottom: 40}} width={350} height={300} data={loaderData.dataset}>
                                        <defs>
                                            <linearGradient id="color" x1="0" y1="0" x2="0" y2="1">
                                                <stop offset="5%" stopColor="#8884d8" stopOpacity={0.3}/>
                                                <stop offset="95%" stopColor="#8884d8" stopOpacity={0}/>
                                            </linearGradient>
                                        </defs>
                                        <Area type="monotone" dataKey="temp" stroke="#8884d8" fill="url(#color)" />
                                        <XAxis padding={{right: 4}} interval={'preserveEnd'} dataKey="label" tick={<CustomizedAxisTick />} />
                                        <YAxis
                                            fontSize="small"
                                            type="number"
                                            unit=" ??C"
                                            padding={{bottom: 20, top: 10}}
                                            tick={{fill: color}}
                                            mirror
                                            domain={[domainMin, domainMax]}
                                        />

                                        <Tooltip content={CustomTooltip} />
                                    </AreaChart>
                                );
                            }
                        }

                    }
                </ClientOnly>

            </div>


        </div>
    );
};




export default TempLog;
