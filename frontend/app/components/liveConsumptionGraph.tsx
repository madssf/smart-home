import React from 'react';
import type {LiveConsumption} from "~/routes/types";
import {Area, AreaChart, Tooltip, YAxis} from "recharts";
import {Alert, AlertDescription} from "~/components/ui/alert";

export interface LiveConsumptionGraphProps {
    liveConsumption?: LiveConsumption[]
}

const LiveConsumptionGraph = ({liveConsumption}: LiveConsumptionGraphProps) => {

    if (liveConsumption === undefined || liveConsumption.length === 0) {
        return <Alert variant="destructive">
            <AlertDescription>
            No consumption data
            </AlertDescription>
        </Alert>;
    }

    const domainMin = Math.round(liveConsumption.reduce((a, b) => b.power < a ? b.power : a, Infinity));
    const domainMax = Math.round(liveConsumption.reduce((a, b) => b.power > a ? b.power : a, 0));

    // TODO: Re-enable dark mode
    // const color = colorMode === 'dark' ? '#F7FAFC' : '#4A5568';
    const color = '#4A5568';

    function CustomTooltip({ active, payload }: any) {
        if (active && payload && payload.length) {
            return (
                <div className="custom-tooltip">
                    <p className="label">{`${payload[0].value} W`}</p>
                </div>
            );
        }

        return null;
    }

    return (
        <AreaChart width={350} height={60} data={liveConsumption}>
            <defs>
                <linearGradient id="color" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#8884d8" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#8884d8" stopOpacity={0}/>
                </linearGradient>
            </defs>
            <Area type="monotone" dataKey="power" stroke="#8884d8" fill="url(#color)" />
            <YAxis
                allowDecimals={false}
                type="number"
                unit=" W"
                tick={{fill: color}}
                mirror
                fontSize="small"
                padding={{bottom: 5, top: 5}}
                interval="preserveStartEnd"
                domain={[domainMin, domainMax]}
            />

            <Tooltip content={CustomTooltip} />
        </AreaChart>
    );
};

export default LiveConsumptionGraph;
