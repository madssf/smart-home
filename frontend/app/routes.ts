export const routes = {
    HOME: '/',
    ROOMS: {
        ROOT: '/rooms',
    },
    SCHEDULES: {
        ROOT: '/schedules',
    },
    PLUGS: {
        ROOT: '/plugs',
    },
    BUTTONS: {
        ROOT: '/buttons',
    },
    TEMP_ACTIONS: {
        ROOT: '/temp_actions',
    },
    TEMP_LOG: {
        ROOT: '/temp_log',
        ROOM_ID: (room_id: string) => `/temp_log/${room_id}`,
    },
    TEMP_SENSORS: {
        ROOT: '/temp_sensors',
    },
    NOTIFICATIONS: {
        ROOT: '/notifications',
    },
};
