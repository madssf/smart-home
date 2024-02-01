import {EnrichedRoomData} from "~/routes/types";
import {Alert, AlertDescription} from "~/components/ui/alert";
import {Link} from "@remix-run/react";
import {routes} from "~/routes";
import {Badge} from "~/components/ui/badge";
import {formatNumber} from "~/utils/formattingUtils";
import dayjs from "dayjs";

type Props = {
    rooms: EnrichedRoomData[]
    hideUnscheduledRooms: boolean
}
const FrontPageRooms = ({rooms, hideUnscheduledRooms}: Props) => {
    if (rooms.length === 0) {
        const message = hideUnscheduledRooms ? 'No rooms with schedule' : 'No rooms added yet';
        return <Alert className="mt-2">
            <AlertDescription>
                {message}
            </AlertDescription>
        </Alert>;
    } else {
        return rooms.sort((a, b) => a.name.localeCompare(b.name)).map((room) => {
            return<FrontPageRoom key={room.id} room={room}/>
        });
    }
};

const FrontPageRoom = ({room}: { room: EnrichedRoomData }) => {
    return (
        <div className="ml-1 mb-1">
            <div className="flex flex-row items-center">
                <Link
                    className="text-xl font-bold cursor-pointer"
                    to={routes.TEMP_LOG.ROOM_ID(room.id)}
                >
                    {room.name}
                </Link>
                {
                    room.temp &&
                    <div className="ml-2 grid grid-cols-[65px_auto] gap-1 p-1 items-end">
                        <Badge className="text-left w-max">
                            {`${formatNumber(room.temp.temp, 1, 1)} °C`}
                        </Badge>
                        <p className="ml-2 mb-0 text-sm">{dayjs(room.temp.time).fromNow()}</p>
                    </div>
                }
            </div>
            <div className="ml-1">
                <div className="grid grid-cols-[70px_auto] gap-1 p-1 items-baseline">
                    <p className="">Schedule</p>
                    {room.activeSchedule?.schedule && room.activeSchedule.temp ?
                        <Badge variant="secondary" className="text-left w-max">
                            {`${formatNumber(room.activeSchedule.temp, 1, 1)} °C`}
                        </Badge>
                        : <Badge
                            className="max-w-max ml-1"
                            variant="outline"
                        >
                            Off
                        </Badge>
                    }
                </div>
                {
                    room.plugStatuses.length > 0 &&
                    <div className="ml-1">
                        <p>Plugs</p>
                        {
                            room.plugStatuses.sort((a, b) => a.name.localeCompare(b.name)).map((plugStatus) => {
                                return (
                                    <div key={plugStatus.name} className="grid grid-cols-[100px_auto] gap-1 p-1 text-sm">
                                        <p>{plugStatus.name}</p>
                                        <Badge
                                            className="max-w-max ml-1"
                                            variant={
                                                plugStatus.is_on === null || plugStatus.power === null ?
                                                    'destructive' :
                                                    plugStatus.is_on ? 'default' : 'secondary'
                                            }
                                        >
                                            {
                                                plugStatus.is_on === null || plugStatus.power === null ?
                                                    'ERROR'
                                                    :
                                                    plugStatus.is_on ?
                                                        `${formatNumber(plugStatus.power, 1, 1)} W`
                                                        : 'OFF'
                                            }
                                        </Badge>
                                    </div>
                                );
                            })
                        }
                    </div>
                }

            </div>
        </div>
    );
}

export default FrontPageRooms;
