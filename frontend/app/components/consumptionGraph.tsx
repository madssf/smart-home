import React from 'react';
import type {Consumption} from "~/routes/types";
import {Area, AreaChart, Tooltip, XAxis, YAxis} from "recharts";
import {useColorMode} from "@chakra-ui/react";

export interface ConsumptionGraphProps {
    consumption: Consumption[]
}

const ConsumptionGraph = ({consumption}: ConsumptionGraphProps) => {

    const domainMax = Math.round(consumption.reduce((a, b) => b.kwh > a ? b.kwh : a, 0)) + 1;

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
                    <p className="label">{`${label} : ${payload[0].value} kWh`}</p>
                </div>
            );
        }

        return null;
    }

    return (
        <AreaChart margin={{bottom: 20}} width={350} height={200} data={consumption}>
            <defs>
                <linearGradient id="color" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#8884d8" stopOpacity={0.8}/>
                    <stop offset="95%" stopColor="#8884d8" stopOpacity={0}/>
                </linearGradient>
            </defs>
            <Area type="monotone" dataKey="kwh" stroke="#8884d8" fillOpacity={1} fill="url(#color)" />
            <XAxis
                padding={{right: 4}}
                interval="preserveEnd"
                dataKey="label"
                tick={<CustomizedAxisTick />}
            />
            <YAxis
                fontSize="small"
                allowDecimals={false}
                type="number"
                unit=" kWh"
                padding={{top: 10}}
                tick={{fill: color}}
                mirror
                domain={[0, domainMax]}
            />

            <Tooltip content={CustomTooltip} />
        </AreaChart>
    );
};

export default ConsumptionGraph;
