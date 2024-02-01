import type {LoaderFunction} from "@remix-run/node";
import {defer} from "@remix-run/node";
import {Await, useLoaderData, useNavigate, useParams, useSearchParams} from "@remix-run/react";
import {Area, AreaChart, Tooltip, XAxis, YAxis} from 'recharts';
import {ClientOnly} from "remix-utils/client-only";
import {routes} from "~/routes";
import {capitalizeAndRemoveUnderscore} from "~/utils/formattingUtils";
import {getRoomTemperatureLogs} from "~/routes/temp_log/temp_log.server";
import {Button} from "~/components/ui/button";
import {useTheme} from "remix-themes";
import {Suspense} from "react";
import {Thermometer} from "lucide-react";

type DatasetEntry = {label: string, temp: number}

type ResponseData = {
    dataset: Promise<DatasetEntry[]>
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

    const tempLogs = getRoomTemperatureLogs(room_id, period);

    return defer<ResponseData>({dataset: tempLogs});

};

export const handle = {hydrate: true};

const TempLog = () => {
    const {room_id} = useParams();
    const {
        dataset
    } = useLoaderData<typeof loader>();
    const [theme] = useTheme()

    const [searchParams] = useSearchParams();
    const navigate = useNavigate();


    const color = theme === 'dark' ? '#F7FAFC' : '#4A5568';
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
                    <p className="label">{`${label} : ${payload[0].value} °C`}</p>
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
                        () => <Suspense fallback={<LoadingComponent/>}>
                            <Await resolve={dataset}>
                                {
                                    (dataset) =>
                                    {
                                        if (dataset.length === 0) {
                                            return <p> No temperature data here..!</p>;
                                        } else {
                                            return (
                                                <AreaChart margin={{bottom: 40}} width={350} height={300} data={dataset}>
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
                                                        unit=" °C"
                                                        padding={{bottom: 20, top: 10}}
                                                        tick={{fill: color}}
                                                        mirror
                                                        domain={getDomain(dataset)}
                                                    />

                                                    <Tooltip content={CustomTooltip} />
                                                </AreaChart>
                                            );
                                        }
                                    }
                                }
                            </Await>
                        </Suspense>
                    }
                </ClientOnly>

            </div>


        </div>
    );
};

const getDomain = (dataset: DatasetEntry[]) => {
    const domainMin = Math.round(dataset.reduce((a, b) => b.temp < a ? b.temp : a, Infinity)) - 3;
    const domainMax = Math.round(dataset.reduce((a, b) => b.temp > a ? b.temp : a, 0)) + 3;
    return [domainMin, domainMax];
}


export default TempLog;


const LoadingComponent = () => {
    return (
        <div className="flex flex-col items-center justify-center space-y-2">
            <Thermometer className="w-12 h-12 animate-spin"/>
            <p className="text-sm text-gray-500">Loading temperature data...</p>
        </div>
    );
};
