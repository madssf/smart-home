
export type NewRequest<T> = Omit<T, 'id'>
export const BASE_URL =  process.env.NODE_ENV === 'production' ? "http://raspi-rust-api:8080/" : "http://127.0.0.1:8080/";

export async function getRequest<T>(endpoint: string): Promise<T> {
    const response = await fetch(
        `${BASE_URL}${endpoint}`,
        {
            method: 'get',
            headers: {
                'Content-Type': 'application/json',
            },
        },
    );
    if (!response.ok) {
        throw new Error("Failed to load data");
    }
    return await response.json();
}

export async function createRequest<T>(endpoint: string, data: T): Promise<void> {

    const response = await fetch(
        `${BASE_URL}${endpoint}`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        },
    );

    if (!response.ok) {
        if (response.status === 400) {
            const message = await response.text();
            throw new Error(message);
        }
        throw new Error("Failed to create");
    }
    return;
}

export async function updateRequest<T extends {id: string}>(endpoint: string, data: T): Promise<void> {
    const {id, ...rest} = data;
    const response = await fetch(
        `${BASE_URL}${endpoint}${id}`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(rest),
        },
    );

    if (!response.ok) {
        throw new Error("Failed to update");
    }
    return;
}

export async function deleteRequest(endpoint: string, id: string): Promise<void> {
    const response = await fetch(
        `${BASE_URL}${endpoint}${id}`,
        {
            method: 'delete',
        },
    );

    if (!response.ok) {
        throw new Error("Failed to delete");
    }
    return;
}
