import React, {useState} from 'react';
import {routes} from "~/routes";
import type {Plug} from "~/routes/plugs/types";
import type {FormErrors} from "~/utils/types";
import type {ActionFunctionArgs, LoaderFunction} from "@remix-run/node";
import {json, redirect} from "@remix-run/node";
import {Link, useLoaderData} from "@remix-run/react";
import {validateIpAddress, validateNonEmptyList, validateNonEmptyString} from "~/utils/validation";
import {piTriggerRefresh} from "~/utils/piHooks";
import {getPlugs} from "~/routes/plugs/plugs.server";
import type {ButtonType} from "~/routes/buttons/types";
import {createButton, deleteButton, getButtons, updateButton} from "~/routes/buttons/buttons.server";
import ButtonForm from "~/routes/buttons/components/buttonForm";
import {Button} from "~/components/ui/button";

interface ResponseData {
    buttons: ButtonType[];
    plugs: Plug[];
}

export type ButtonFormErrors = FormErrors<ButtonType>;

export const handle = {hydrate: true};

export async function action({request}: ActionFunctionArgs) {

    const body = await request.formData();

    const id = body.get("id")?.toString();
    const name = body.get("name")?.toString();
    const ip = body.get("ip")?.toString();
    const username = body.get("username")?.toString();
    const password = body.get("password")?.toString();
    const plugIds = body.getAll("plug_id")?.map(p => p.toString());


    const intent = body.get("intent")?.toString();

    if (intent === 'delete') {
        await deleteButton(id!);
        await piTriggerRefresh();
        return redirect(routes.BUTTONS.ROOT);
    }

    const validated = {
        name: validateNonEmptyString(name),
        ip: validateIpAddress(ip),
        username: validateNonEmptyString(username),
        password: validateNonEmptyString(password),
        plugIds: validateNonEmptyList(plugIds),
    };

    if (!validated.name.valid || !validated.ip.valid || !validated.username.valid || !validated.password.valid || !validated.plugIds.valid) {
        return json<ButtonFormErrors>(
            {
                id,
                name: !validated.name.valid ? validated.name.error : undefined,
                ip: !validated.ip.valid ? validated.ip.error : undefined,
                username: !validated.username.valid ? validated.username.error : undefined,
                password: !validated.password.valid ? validated.password.error : undefined,
                plug_ids: !validated.plugIds.valid ? validated.plugIds.error : undefined,
            },
        );
    }

    const document: Omit<ButtonType, 'id'> = {
        name: validated.name.data,
        ip: validated.ip.data,
        username: validated.username.data,
        password: validated.password.data,
        plug_ids: validated.plugIds.data,
    };

    if (!id) {
        await createButton(document);
    } else {
        await updateButton({...document, id});
    }
    await piTriggerRefresh();
    return redirect(routes.BUTTONS.ROOT);
}

export const loader: LoaderFunction = async () => {

    const plugs = await getPlugs();
    const buttons = await getButtons();

    return json<ResponseData>({
        plugs,
        buttons,
    });

};

const Buttons = () => {

    const loaderData = useLoaderData<ResponseData>();
    const [showNew, setShowNew] = useState(false);

    const renderButtons = (buttons: ButtonType[]) => {
        return buttons.map((button) => {
            return (
                <ButtonForm key={button.id} button={button} plugs={loaderData.plugs}/>
            );
        });
    };

    return (
        <div>
            <h1 className="pb-4">Buttons</h1>
            {
                loaderData.plugs.length === 0 ?
                    <p>No plugs yet, please <Link to={routes.PLUGS.ROOT}>add one</Link> before adding a button</p>
                    :
                    <>
                        {renderButtons(loaderData.buttons)}
                        <Button className="my-1" onClick={() => setShowNew((prev) => (!prev))}>{showNew ? 'Cancel' : 'Add button'}</Button>
                        {
                            showNew &&
                            <ButtonForm plugs={loaderData.plugs} />
                        }
                    </>
            }

        </div>
    );
};

export default Buttons;
