import {NaiveTime, PRICE_LEVELS, PriceLevel, TimeWindow, Weekday, WEEKDAYS} from "~/routes/schedules/types/types";
import {Validate} from "~/utils/types";

export const validatePriceLevel = (priceLevel?: string): Validate<PriceLevel> => {
    if (!priceLevel) {
        return {valid: false, error: "Price level is required"}
    }
    if (!PRICE_LEVELS.includes(priceLevel as PriceLevel)) {
        return {valid: false, error: "Unknown price level"}
    }
    return {
        valid: true,
        data: priceLevel as PriceLevel
    }
}

export const validateDays = (days: string[]): Validate<Weekday[]> => {
    if (days.length === 0) {
        return {valid: false, error: "Minimum one day is required"}
    }
    if (days.some((day) => !WEEKDAYS.includes(day as Weekday))) {
        return {valid: false, error: "Unknown weekday"}
    }
    return {
        valid: true,
        data: [...new Set([...days.map((day) => day as Weekday)])]
    }
}

export const validateHours = (from: string[], to: string[]): Validate<TimeWindow[]> => {
    if (from.length === 0 || to.length === 0 || from.length !== to.length) {
        return {valid: false, error: "Invalid time windows"}
    }
    const validated: TimeWindow[] = [];
    for (let i = 0; i < from.length; i++) {
        const fromValidated = toNaiveTime(from[i])
        if (!fromValidated.valid) {
            return fromValidated
        }
        const toValidated = toNaiveTime(to[i])
        if (!toValidated.valid) {
            return toValidated
        }
        validated.push({
            from: fromValidated.data,
            to: toValidated.data
        })
    }
    return {valid: true, data: validated}

}

const toNaiveTime = (str: string): Validate<NaiveTime> => {
    if (str.length !== 5 || str[2] !== ':') {
        return {valid: false, error: "Invalid time, should be HH:MM on 24h format"}
    }
    const hour = Number(str.slice(0, 2))
    const min = Number(str.slice(3, 5))
    if (!Number.isInteger(hour) || !Number.isInteger(min) || hour < 0 || hour > 23 || min < 0 || min > 59) {
        return {valid: false, error: "Invalid time, should be HH:MM on 24h format"}
    }
    return {
        valid: true,
        data: `${String(hour).padStart(2,'0')}:${String(min).padStart(2, '0')}`
    }
}
