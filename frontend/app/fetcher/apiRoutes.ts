export const apiRoutes = {
    rooms: 'rooms/',
    plugs: 'plugs/',
    schedules: 'schedules/',
    temp_actions: 'temp_actions/',
    temperature_logs: (room_id: string) => `temperature_logs/${room_id}`,
    prices: {
        current: 'prices/current',
    },
};
