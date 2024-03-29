import {routes} from "~/routes";
import type {FormErrors} from "~/utils/types";
import type {ActionFunctionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import {useLoaderData} from "@remix-run/react";
import type {TempAction} from "~/routes/temp_actions/types";
import {
    validateActionType,
    validateDateTime,
    validateDateTimeOrNull,
    validateNonEmptyList,
    validateTempOrNull,
} from "~/utils/validation";
import TempActionForm from "~/routes/temp_actions/components/tempActionForm";
import {piTriggerRefresh} from "~/utils/piHooks";
import {
    createTempAction,
    deleteTempAction,
    getTempActions,
    updateTempAction,
} from "~/routes/temp_actions/tempActions.server";
import {getRooms} from "~/routes/rooms/rooms.server";
import type {Room} from "~/routes/rooms/types";

interface ResponseData {
    tempActions: TempAction[];
    rooms: Room[];
}

export type TempActionErrors = FormErrors<TempAction>;

export const handle = {hydrate: true};

export async function action({request}: ActionFunctionArgs) {

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const roomIds = body.getAll("room_ids").map((room_id) => room_id.toString());
    const actionType = body.get("actionType")?.toString();
    const temp = body.get("temp")?.toString();
    const startsAtDate = body.get("startsAt-date")?.toString();
    const startsAtTime = body.get("startsAt-time")?.toString();
    const expiresAtDate = body.get("expiresAt-date")?.toString();
    const expiresAtTime = body.get("expiresAt-time")?.toString();

    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deleteTempAction(id!);
        await piTriggerRefresh();
        return redirect(routes.TEMP_ACTIONS.ROOT);
    }

    const validated = {
        roomIds: validateNonEmptyList(roomIds),
        actionType: validateActionType(actionType),
        temp: validateTempOrNull(temp),
        expiresAt: validateDateTime(expiresAtDate, expiresAtTime),
        startsAt: validateDateTimeOrNull(startsAtDate, startsAtTime),
    };


    if (!validated.roomIds.valid || !validated.temp.valid || !validated.actionType.valid || !validated.expiresAt.valid || !validated.startsAt.valid) {
        return json<TempActionErrors>(
            {
                id,
                room_ids: validated.roomIds.error,
                action: validated.actionType.error,
                temp: validated.temp.error,
                expires_at: validated.expiresAt.error,
                starts_at: validated.startsAt.error,
            },
        );
    }


    const document: Omit<TempAction, 'id'> = {
        room_ids: validated.roomIds.data,
        action: validated.actionType.data,
        temp: validated.temp.data,
        expires_at: validated.expiresAt.data,
        starts_at: validated.startsAt.data,
    };

    if (!id) {
        await createTempAction(document);
    } else {
        await updateTempAction({id, ...document});
    }

    await piTriggerRefresh();

    return redirect(routes.TEMP_ACTIONS.ROOT);
}

export const loader: LoaderFunction = async () => {

    const rooms = await getRooms();
    const tempActions = await getTempActions();

    return json<ResponseData>({
        tempActions,
        rooms,
    });

};

const TempActions = () => {

    const loaderData = useLoaderData<ResponseData>();

    const renderTempActions = (tempActions: TempAction[], rooms: Room[]) => {
        return tempActions.map((tempAction) => {
            return (
                <TempActionForm key={tempAction.id} tempAction={tempAction} rooms={rooms} />
            );
        });
    };

    return (
        <div>
            <h1 className="pb-4">Actions</h1>
            {renderTempActions(loaderData.tempActions, loaderData.rooms)}
            <TempActionForm rooms={loaderData.rooms} />
        </div>
    );
};

export default TempActions;
