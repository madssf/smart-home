import React from 'react';
import type {TimeWindow} from "~/routes/schedules/types";
import {Input} from "@chakra-ui/input";
import {Button} from "@chakra-ui/react";
import {SmallAddIcon, SmallCloseIcon} from "@chakra-ui/icons";

export interface TimeFormProps {
    window: TimeWindow;
    handleRemove: () => void;
    handleAdd?: () => void;
}

const TimeForm = ({window, handleRemove, handleAdd}: TimeFormProps) => {
    return (
        <>
            <div className="flex flex-row items-baseline">
                <label className="mr-1">From</label>
                <Input style={{maxWidth: "5rem"}} name="from" defaultValue={window[0].slice(0, 5)}/>
                <label className="mx-1">To</label>
                <Input style={{maxWidth: "5rem"}} name="to" defaultValue={window[1].slice(0, 5)}/>
                <Button className="mx-1" size="sm" variant="outline" type="button" onClick={() => {
                    handleRemove();
                }}>
                    <SmallCloseIcon/>
                </Button>
                {
                    handleAdd !== undefined &&
                    <Button className="mb-1" size="sm" type="button" onClick={handleAdd}><SmallAddIcon /></Button>
                }

            </div>

        </>
    );
};

export default TimeForm;
