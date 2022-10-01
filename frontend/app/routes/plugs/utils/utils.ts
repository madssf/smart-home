import {Validate} from "~/utils/types";

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
