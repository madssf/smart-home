import * as React from 'react';

export interface DatePickerProps {
    name: string,
    defaultValue?: string
}

function DatePicker({name, defaultValue}: DatePickerProps) {

    return (
        <>
        <input className="mr-2 min-w-max" type='date' name={`${name}-date`} defaultValue={defaultValue?.slice(0, 10)}/>
        <input className="min-w-max" type='time' name={`${name}-time`} defaultValue={defaultValue?.slice(11, 16)}/>
        </>
    );
}

export default DatePicker;
