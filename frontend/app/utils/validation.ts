import {Validate} from "~/utils/types";
import {ActionType} from "~/routes/temp_actions/types";

export const validateIpAddress = (str?: string): Validate<string> => {
    if (!str) {
        return {valid: false, error: 'IP address is required'}
    }
    const regex = /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/
    return regex.test(str) ?
        {valid: true, data: str} : {valid: false, error: "Not a valid IPv4 address"}
}

export const validateNonEmptyString = (str?: string): Validate<string> => {
    return (!!str && str.length > 0) ? {valid: true, data: str} : {valid: false, error: 'Required'}
}

export const validateNonEmptyList = (list?: string[]): Validate<string[]> => {
    return (!!list && !list.some((element) => element.length === 0)) ? {valid: true, data: list} : {valid: false, error: 'Can\'t be empty'}
}

export const validateActionType = (str?: string): Validate<ActionType> => {
    if (!!str && Object.values(ActionType).some((value) => value === str)) {
        return {valid: true, data: str as ActionType}
    }
    return {valid: false, error: "Must be ON or OFF"}
}
