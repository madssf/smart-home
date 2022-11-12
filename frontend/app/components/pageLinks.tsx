import {Link} from "@chakra-ui/react";
import {routes} from "~/routes";

const homeLink = <Link className="my-2 text-xl" key={routes.HOME} href={routes.HOME}>Home</Link>;

const pageLinks = [
    <Link className="my-2 text-xl" key={routes.ROOMS.ROOT} href={routes.ROOMS.ROOT}>Rooms</Link>,
    <Link className="my-2 text-xl" key={routes.PLUGS.ROOT} href={routes.PLUGS.ROOT}>Plugs</Link>,
    <Link className="my-2 text-xl" key={routes.SCHEDULES.ROOT} href={routes.SCHEDULES.ROOT}>Schedules</Link>,
    <Link className="my-2 text-xl" key={routes.TEMP_ACTIONS.ROOT} href={routes.TEMP_ACTIONS.ROOT}>Temp actions</Link>,
    <Link className="my-2 text-xl" key={routes.TEMP_LOG.ROOT} href={routes.TEMP_LOG.ROOT}>Temperatures</Link>,
    <Link className="my-2 text-xl" key={routes.NOTIFICATIONS.ROOT} href={routes.NOTIFICATIONS.ROOT}>Notifications</Link>,
];

export {homeLink, pageLinks};
