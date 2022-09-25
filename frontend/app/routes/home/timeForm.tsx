import React from 'react';
import {NaiveTime, TimeWindow} from "~/routes/home/types";

export interface TimeFormProps {
    window?: TimeWindow;
    handleRemove: () => void;
}

const TimeForm = ({window, handleRemove}: TimeFormProps) => {
    return (
        <div>
            <label className="font-bold mr-2">From</label><input name="from" defaultValue={window?.from}/>
            <label className="font-bold mr-2">To</label><input name="to" defaultValue={window?.to}/>
            <button type="button" onClick={() => {
                handleRemove()
            }}>Remove</button>
        </div>
    );
};

export default TimeForm;