import React from 'react';
import type {Consumption} from "~/routes/types";
import {ClientOnly} from "remix-utils";
import {Line, LineChart, Tooltip, XAxis, YAxis} from "recharts";
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
                                <LineChart margin={{bottom: 40}} width={360} height={300} data={consumption}>
                                    <Line type="monotone" dataKey="kwh" stroke="#8884d8" strokeWidth={1.5} />
                                    <XAxis padding={{right: 4}} interval={'preserveEnd'} dataKey="label" tick={<CustomizedAxisTick />} />
                                    <YAxis type="number" padding={{bottom: 40}} tick={{fill: color}} mirror domain={[0, domainMax]} />

                                    <Tooltip content={CustomTooltip} />
                                </LineChart>
                            );
                        }
                    }

                }
            </ClientOnly>

        </div>
    );
};

export default ConsumptionGraph;
