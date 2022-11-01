import React from 'react';
import type {Consumption} from "~/routes/types";
import {ClientOnly} from "remix-utils";
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
                <text x={12} y={0} dy={16} textAnchor="end" fill={color} transform="rotate(-35)">
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
        <div className="flex">
            <ClientOnly>
                {
                    () =>
                    {
                        if (consumption.length === 0) {
                            return <p>No consumption data</p>;
                        } else {
                            return (
                                <AreaChart margin={{bottom: 40}} width={350} height={300} data={consumption}>
                                    <defs>
                                        <linearGradient id="color" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="#8884d8" stopOpacity={0.8}/>
                                            <stop offset="95%" stopColor="#8884d8" stopOpacity={0}/>
                                        </linearGradient>
                                    </defs>
                                    <Area type="monotone" dataKey="kwh" stroke="#8884d8" fillOpacity={1} fill="url(#color)" />
                                    <XAxis padding={{right: 4}} interval="preserveEnd" dataKey="label" tick={<CustomizedAxisTick />} />
                                    <YAxis
                                        type="number"
                                        unit=" kWh"
                                        padding={{bottom: 40}}
                                        tick={{fill: color}}
                                        mirror
                                        domain={[0, domainMax]}
                                    />

                                    <Tooltip content={CustomTooltip} />
                                </AreaChart>
                            );
                        }
                    }

                }
            </ClientOnly>

        </div>
    );
};

export default ConsumptionGraph;
