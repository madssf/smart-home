import React from 'react';
import {TimeWindow} from "~/routes/schedules/types/types";
import {Input} from "@chakra-ui/input";
import {Button} from "@chakra-ui/react";

export interface TimeFormProps {
    window?: TimeWindow;
    handleRemove: () => void;
}

const TimeForm = ({window, handleRemove}: TimeFormProps) => {
    return (
        <div>
            <label className="font-bold mr-2">From</label><Input name="from" defaultValue={window?.from}/>
            <label className="font-bold mr-2">To</label><Input name="to" defaultValue={window?.to}/>
            <Button size="sm" variant="outline" type="button" onClick={() => {
                handleRemove()
            }}>Remove</Button>
        </div>
    );
};

export default TimeForm;