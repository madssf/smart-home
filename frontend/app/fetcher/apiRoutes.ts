import type {TimePeriod} from "~/routes/temp_log/$room_id";

export const apiRoutes = {
    rooms: 'roomsa/',
    plugs: 'plugs/',
    schedules: 'schedules/',
    temp_actions: 'temp_actions/',
    temperature_logs: (room_id: string, time_period: TimePeriod) => `temperature_logs/${room_id}/${time_period}`,
    prices: {
        current: 'prices/current',
    },
};
