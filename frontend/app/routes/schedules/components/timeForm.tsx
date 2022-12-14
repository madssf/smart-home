import React from 'react';
import type {TimeWindow} from "~/routes/schedules/types";
import {Button, IconButton} from "@chakra-ui/react";
import {SmallAddIcon, SmallCloseIcon} from "@chakra-ui/icons";

export interface TimeFormProps {
    window: TimeWindow;
    handleRemove: () => void;
    handleAdd?: () => void;
}

const TimeForm = ({window, handleRemove, handleAdd}: TimeFormProps) => {
    return (
        <>
            <div className="grid grid-cols-[50px_80px_40px_80px_40px_30px] items-center">
                <label className="mr-1">From</label>
                <input className="w-20" type='time' name="from" defaultValue={window[0].slice(0, 5)}/>
                <label className="ml-2 mr-1">To</label>
                <input className="w-20" type='time' name="to" defaultValue={window[1].slice(0, 5)}/>
                <Button className="mx-1" size="sm" variant="outline" type="button" onClick={() => {
                    handleRemove();
                }}>
                    <SmallCloseIcon/>
                </Button>
                {
                    handleAdd !== undefined &&
                    <IconButton icon={<SmallAddIcon />} aria-label='Add time window' size="sm" type="button" onClick={handleAdd} />
                }

            </div>

        </>
    );
};

export default TimeForm;
