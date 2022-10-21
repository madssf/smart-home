export const routes = {
    HOME: '/home',
    ROOMS: {
        ROOT: '/rooms',
    },
    SCHEDULES: {
        ROOT: '/schedules',
    },
    PLUGS: {
        ROOT: '/plugs',
    },
    TEMP_ACTIONS: {
        ROOT: '/temp_actions',
    },
    TEMP_LOG: {
        ROOT: '/temp_log',
        ROOM_ID: (room_id: string) => `/temp_log/${room_id}`,
    },
};
