import type {Plug} from "~/routes/plugs/types/types";
import type {EmptyResponse} from "~/utils/fetcher";


export async function getPlugs(): Promise<Plug[]> {
    const response = await fetch(
        "http://raspi-rust-api:8080/plugs/",
        {
            method: 'get',
            headers: {
                'Content-Type': 'application/json',
            },
        },
    );

    if (!response.ok) {
        throw new Error("Couldn't load plugs");

    }
    return await response.json();
}

export async function createPlug(plug: Omit<Plug, 'id'>): Promise<EmptyResponse> {
    const response = await fetch(
        "http://raspi-rust-api:8080/plugs/",
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(plug),
        },
    );

    if (!response.ok) {
        return 'ERROR';
    } else {
        return 'OK';
    }
}

export async function updatePlug(plug: Plug): Promise<EmptyResponse> {
    const {id, ...rest} = plug;
    const response = await fetch(
        `http://raspi-rust-api:8080/plugs/${id}`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(rest),
        },
    );

    if (!response.ok) {
        return 'ERROR';
    } else {
        return 'OK';
    }
}

export async function deletePlug(id: string): Promise<EmptyResponse> {
    const response = await fetch(
        `http://raspi-rust-api:8080/plugs/${id}`,
        {
            method: 'delete',
        },
    );

    if (!response.ok) {
        return 'ERROR';
    } else {
        return 'OK';
    }
}
