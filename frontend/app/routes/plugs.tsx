import React, {useState} from 'react';
import {routes} from "~/routes";
import {Plug} from "~/routes/plugs/types/types";
import {FormErrors} from "~/utils/types";
import {ActionArgs, json, LoaderFunction, redirect} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {db} from "~/utils/firebase.server";
import {collections} from "~/utils/firestoreUtils.server";
import PlugForm from "~/routes/plugs/components/plugForm";
import {useLoaderData} from "@remix-run/react";
import {Button} from "@chakra-ui/react";
import {validateIpAddress, validateNonEmptyString} from "~/utils/validation";
import {useTriggerRefresh} from "~/utils/raspiHooks";

interface ResponseData {
    plugs: Plug[];
}

export type PlugFormErrors = FormErrors<Plug>;

export const handle = {hydrate: true};

export async function action({request}: ActionArgs) {

    const {userId} = await requireUserId(request)

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const name = body.get("name")?.toString();
    const ip = body.get("ip")?.toString();
    const username = body.get("username")?.toString();
    const password = body.get("password")?.toString();

    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await db.doc(`${collections.plugs(userId)}/${id}`).delete().catch((e) => {throw Error("Something went wrong")})
        await useTriggerRefresh();
        return redirect(routes.PLUGS.ROOT)
    }

    const validated = {
        name: validateNonEmptyString(name),
        ip: validateIpAddress(ip),
        username: validateNonEmptyString(username),
        password: validateNonEmptyString(password),
    }

    if (!validated.name.valid || !validated.ip.valid || !validated.username.valid || !validated.password.valid) {
        return json<PlugFormErrors>(
            {
                id,
                name: !validated.name.valid ? validated.name.error : undefined,
                ip: !validated.ip.valid ? validated.ip.error : undefined,
                username: !validated.username.valid ? validated.username.error : undefined,
                password: !validated.password.valid ? validated.password.error : undefined,
            }
        )
    }

    const document: Omit<Plug, 'id'> = {
        name: validated.name.data, ip: validated.ip.data, username: validated.username.data, password: validated.password.data,
    }

    if (!id) {
        await db.collection(collections.plugs(userId)).add(document).catch((e) => {throw Error("Something went wrong")})
    } else {
        await db.doc(`${collections.plugs(userId)}/${id}`).set(document).catch((e) => {throw Error("Something went wrong")})
    }
    await useTriggerRefresh();
    return redirect(routes.PLUGS.ROOT);
}

export const loader: LoaderFunction = async ({request}) => {

    const {userId} = await requireUserId(request)

    const plugsRef = await db.collection(collections.plugs(userId)).get()
    const plugs = plugsRef.docs.map((doc) => {
        const data = doc.data()
        // TODO: Validate
        const plug: Plug = {
            id: doc.id, name: data.name, ip: data.ip, username: data.username, password: data.password
        }
        return plug
    })

    return json<ResponseData>({
        plugs,
    });

};

const Plugs = () => {

    const loaderData = useLoaderData<ResponseData>()
    const [showNew, setShowNew] = useState(false)

    const renderPlugs = (plugs: Plug[]) => {
        return plugs.map((plug) => {
            return (
                <PlugForm key={plug.id} plug={plug}/>
            )
        })
    }

    return (
        <div>
            {renderPlugs(loaderData.plugs)}
            <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add plug'}</Button>
            {
                showNew &&
                <PlugForm />
            }
        </div>
    );
};

export default Plugs;
