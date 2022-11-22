import type {Validate} from "~/utils/types";
import {ActionType} from "~/routes/temp_actions/types";

export const validateIpAddress = (str?: string): Validate<string> => {
    if (!str) {
        return {valid: false, error: 'IP address is required'};
    }
    // eslint-disable-next-line max-len
    const regex = /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
    return regex.test(str) ?
        {valid: true, data: str} : {valid: false, error: "Not a valid IPv4 address"};
};

export const validateNonEmptyString = (str?: string): Validate<string> => {
    return (!!str && str.length > 0) ? {valid: true, data: str} : {valid: false, error: 'Required'};
};

export const validateNonEmptyList = (list?: string[]): Validate<string[]> => {
    return (!!list && list.length !== 0 && !list.some((element) => element.length === 0)) ?
        {valid: true, data: list} : {valid: false, error: 'Can\'t be empty'};
};

export const validateActionType = (str?: string): Validate<ActionType> => {
    if (!!str && Object.values(ActionType).some((value) => value === str)) {
        return {valid: true, data: str as ActionType};
    }
    return {valid: false, error: "Must be ON or OFF"};
};

export const validateDateTime = (dateStr?: string, timeStr?: string): Validate<string> => {
    if (!dateStr || dateStr.length !== 10 || !timeStr || timeStr.length !== 5) {
        return {valid: false, error: 'Date and time are required'};
    }
    return {valid: true, data: `${dateStr}T${timeStr}:00`};
};

export const validateTempOrNull = (str?: string): Validate<number | null> => {
    if (str === undefined) {
        return {valid: true, data: null};
    }
    const num = Number(str);
    if (Number.isNaN(num)) {
        return {valid: false, error: 'Not a valid temperature'};
    }
    if (num < 1 || num > 30) {
        return {valid: false, error: 'Temperature must be between 1 and 30 degrees'};
    }
    return {valid: true, data: num};
};

export const validatePositiveNonZeroInteger = (str?: string): Validate<number> => {
    const num = Number(str);
    if (!Number.isInteger(num)) {
        return {valid: false, error: 'Not a valid integer'};
    }
    if (num < 1 ) {
        return {valid: false, error: 'Not a valid positive integer'};
    }
    return {valid: true, data: num};
};
