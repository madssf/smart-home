import {Input} from "~/components/ui/input";

export interface DatePickerProps {
    name: string,
    defaultValue?: string
}

function DatePicker({name, defaultValue}: DatePickerProps) {

    console.log(defaultValue)
    return (
        <div className="flex flex-row">
            <Input className="mr-2 min-w-max" type='date' name={`${name}-date`} defaultValue={defaultValue?.slice(0, 10)}/>
            <Input className="min-w-max" type='time' name={`${name}-time`} defaultValue={defaultValue?.slice(11, 16)}/>
        </div>
    );
}

export default DatePicker;
