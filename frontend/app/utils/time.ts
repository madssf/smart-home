import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import timezone from "dayjs/plugin/timezone";

export const getNow = () => {
    dayjs.extend(utc);
    dayjs.extend(timezone);
    const tz = process.env.TIMEZONE!;
    return dayjs().tz(tz);
};
