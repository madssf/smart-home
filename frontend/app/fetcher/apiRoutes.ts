import type {TimePeriod} from "~/routes/temp_log/$room_id";

export const apiRoutes = {
    rooms: 'rooms/',
    plugs: 'plugs/',
    plug_status: 'plugs/status',
    schedules: 'schedules/',
    active_schedules: 'schedules/active',
    temp_actions: 'temp_actions/',
    temperature_logs: {
        room_id: (room_id: string, time_period: TimePeriod) => `temperature_logs/${room_id}/${time_period}`,
        current: 'temperature_logs/current',
    },
    prices: {
        current: 'prices/current',
        consumption: 'prices/consumption',
        live_consumption: 'prices/live_consumption',
    },
    notification_settings: 'notification_settings/',
};
