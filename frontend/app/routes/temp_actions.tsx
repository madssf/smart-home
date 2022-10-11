import React, {useState} from 'react';
import {routes} from "~/routes";
import {FormErrors} from "~/utils/types";
import {ActionArgs, json, LoaderFunction, redirect} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {db} from "~/utils/firebase.server";
import {collections} from "~/utils/firestoreUtils.server";
import {useLoaderData} from "@remix-run/react";
import {Button} from "@chakra-ui/react";
import {TempAction} from "~/routes/temp_actions/types";
import {validateActionType, validateNonEmptyList, validateNonEmptyString} from "~/utils/validation";
import {Plug} from "~/routes/plugs/types/types";
import TempActionForm from "~/routes/temp_actions/components/tempActionForm";

interface ResponseData {
    tempActions: TempAction[];
    plugs: Plug[];
}

export type TempActionErrors = FormErrors<TempAction>;

export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

    const {userId} = await requireUserId(request)

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const plugIds = body.getAll("plugIds").map((plug_id) => plug_id.toString());
    const actionType = body.get("actionType")?.toString();
    const expiresAt = body.get("expiresAt")?.toString();

    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await db.doc(`${collections.temp_actions(userId)}/${id}`).delete().catch((e) => {throw Error("Something went wrong")})
        return redirect(routes.PLUGS.ROOT)
    }

    const validated = {
        plugIds: validateNonEmptyList(plugIds),
        actionType: validateActionType(actionType),
        expiresAt: validateNonEmptyString(expiresAt),
    }

    console.log(validated)

    if (!validated.plugIds.valid || !validated.actionType.valid || !validated.expiresAt.valid) {
        return json<TempActionErrors>(
            {
                id,
                plug_ids: validated.plugIds.error,
                action_type: validated.actionType.error,
                expires_at: validated.expiresAt.error,
            }
        )
    }


    const document: Omit<TempAction, 'id'> = {
        plug_ids: validated.plugIds.data, action_type: validated.actionType.data, expires_at: validated.expiresAt.data,
    }

    if (!id) {
        await db.collection(collections.temp_actions(userId)).add(document).catch((e) => {throw Error("Something went wrong")})
    } else {
        await db.doc(`${collections.temp_actions(userId)}/${id}`).set(document).catch((e) => {throw Error("Something went wrong")})
    }

    return redirect(routes.TEMP_ACTIONS.ROOT);
}

export const loader: LoaderFunction = async ({request}) => {

    const {userId} = await requireUserId(request)

    const plugsRef = await db.collection(collections.plugs(userId)).get()
    const plugs = plugsRef.docs.map((doc) => {
        const data = doc.data()
        // TODO: Validate and DRY from plugs
        const plug: Plug = {
            id: doc.id, name: data.name, ip: data.ip, username: data.username, password: data.password,
        }
        return plug
    })

    const tempActionsRef = await db.collection(collections.temp_actions(userId)).get()
    const tempActions = tempActionsRef.docs.map((doc) => {
        const data = doc.data()
        // TODO: Validate
        const tempAction: TempAction = {
            id: doc.id, plug_ids: data.plug_ids, action_type: data.action_type, expires_at: data.expires_at
        }
        return tempAction
    })

    return json<ResponseData>({
        tempActions,
        plugs,
    });

};

const TempActions = () => {

    const loaderData = useLoaderData<ResponseData>()
    const [showNew, setShowNew] = useState(false)

    const renderTempActions = (tempActions: TempAction[], plugs: Plug[]) => {
        return tempActions.map((tempAction) => {
            return (
                <TempActionForm key={tempAction.id} tempAction={tempAction} plugs={plugs} />
            )
        })
    }

    return (
        <div>
            {renderTempActions(loaderData.tempActions, loaderData.plugs)}
            <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add temporary action'}</Button>
            {
                showNew &&
                <TempActionForm plugs={loaderData.plugs} />
            }
        </div>
    );
};

export default TempActions;
