import {routes} from "~/routes";

const homeLink = {key: routes.HOME, name: "Home"};

const pageLinks = [
    {key: routes.ROOMS.ROOT, name: "Rooms"},
    {key: routes.PLUGS.ROOT, name: "Plugs"},
    {key: routes.BUTTONS.ROOT, name: "Buttons"},
    {key: routes.TEMP_SENSORS.ROOT, name: "Sensors"},
    {key: routes.SCHEDULES.ROOT, name: "Schedules"},
    {key: routes.TEMP_ACTIONS.ROOT, name: "Actions"},
    {key: routes.TEMP_LOG.ROOT, name: "Temperatures"},
    {key: routes.NOTIFICATIONS.ROOT, name: "Notifications"},
];


export {homeLink, pageLinks};
