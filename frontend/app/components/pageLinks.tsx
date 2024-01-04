import {routes} from "~/routes";
import {Link} from "@remix-run/react";

const homeLink = <Link className="my-2 text-xl" key={routes.HOME} to={routes.HOME}>Home</Link>;

const pageLinks = [
    <Link className="my-2 text-xl" key={routes.ROOMS.ROOT} to={routes.ROOMS.ROOT}>Rooms</Link>,
    <Link className="my-2 text-xl" key={routes.PLUGS.ROOT} to={routes.PLUGS.ROOT}>Plugs</Link>,
    <Link className="my-2 text-xl" key={routes.BUTTONS.ROOT} to={routes.BUTTONS.ROOT}>Buttons</Link>,
    <Link className="my-2 text-xl" key={routes.TEMP_SENSORS.ROOT} to={routes.TEMP_SENSORS.ROOT}>Sensors</Link>,
    <Link className="my-2 text-xl" key={routes.SCHEDULES.ROOT} to={routes.SCHEDULES.ROOT}>Schedules</Link>,
    <Link className="my-2 text-xl" key={routes.TEMP_ACTIONS.ROOT} to={routes.TEMP_ACTIONS.ROOT}>Actions</Link>,
    <Link className="my-2 text-xl" key={routes.TEMP_LOG.ROOT} to={routes.TEMP_LOG.ROOT}>Temperatures</Link>,
    <Link className="my-2 text-xl" key={routes.NOTIFICATIONS.ROOT} to={routes.NOTIFICATIONS.ROOT}>Notifications</Link>,
];

export {homeLink, pageLinks};
