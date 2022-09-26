import React, {useState} from 'react';
import {routes} from "~/routes";
import {Plug} from "~/routes/plugs/types/types";
import {FormErrors} from "~/utils/types";
import {ActionArgs, json, LoaderFunction, redirect} from "@remix-run/node";
import {requireUserId} from "~/utils/sessions.server";
import {db} from "~/utils/firebase.server";
import {collections} from "~/utils/firestoreUtils.server";
import {validateIpAddress, validateName} from "~/routes/plugs/utils/utils";
import PlugForm from "~/routes/plugs/components/plugForm";
import {Outlet, useLoaderData} from "@remix-run/react";
import {Button} from "@chakra-ui/react";

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


    const validated = {
        name: validateName(name),
        ip: validateIpAddress(ip),
    }

    if (!validated.name.valid || !validated.ip.valid) {
        return json<PlugFormErrors>(
            {
                id,
                name: !validated.name.valid ? validated.name.error : undefined,
                ip: !validated.ip.valid ? validated.ip.error : undefined,
            }
        )
    }

    const document: Omit<Plug, 'id'> = {
        name: validated.name.data, ip: validated.ip.data,
    }

    if (!id) {
        await db.collection(collections.plugs(userId)).add(document).catch((e) => {throw Error("Something went wrong")})
    } else {
        await db.doc(`${collections.plugs(userId)}/${id}`).set(document).catch((e) => {throw Error("Something went wrong")})
    }

    return redirect(routes.PLUGS.ROOT);
}

export const loader: LoaderFunction = async ({request}) => {

    const {userId} = await requireUserId(request)

    const plugsRef = await db.collection(collections.plugs(userId)).get()
    const plugs = plugsRef.docs.map((doc) => {
        const data = doc.data()
        // TODO: Validate
        const plug: Plug = {
            name: data.name, ip: data.ip, id: doc.id,
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
            <Button onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add plug'}</Button>
            {
                showNew &&
                <PlugForm />
            }
            <Outlet />
        </div>
    );
};

export default Plugs;