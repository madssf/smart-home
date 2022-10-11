import React from 'react';
import {TimeWindow} from "~/routes/schedules/types/types";
import {Input} from "@chakra-ui/input";
import {Button} from "@chakra-ui/react";
import {CloseIcon} from "@chakra-ui/icons";

export interface TimeFormProps {
    window?: TimeWindow;
    handleRemove: () => void;
}

const TimeForm = ({window, handleRemove}: TimeFormProps) => {
    return (
        <>
            <div>
                    <label className="mr-1">From</label>
                    <Input style={{maxWidth: "5rem"}} name="from" defaultValue={window?.from}/>
                    <label className="mx-1">To</label>
                    <Input style={{maxWidth: "5rem"}} name="to" defaultValue={window?.to}/>
                    <Button className="ml-1" size="sm" variant="outline" type="button" onClick={() => {
                        handleRemove()
                    }}><CloseIcon/></Button>

            </div>

        </>
    );
};

export default TimeForm;
