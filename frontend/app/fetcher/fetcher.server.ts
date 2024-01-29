export type CreateRequest<T> = Omit<T, 'id'>
export type SimpleResult<T> = T | 'ERROR'
export const BASE_URL = process.env.NODE_ENV === 'production' ? "http://raspi-rust-api:8081/" : "http://127.0.0.1:8081/";

export async function getRequest<T>(endpoint: string): Promise<T> {
    const response = await fetchWithRetry(
        `${BASE_URL}${endpoint}`,
        {
            method: 'get',
            headers: {
                'Content-Type': 'application/json',
            },
        },
    );
    if (!response.ok) {
        throw new Error(`Failed to get data - server status: ${response.status}`);
    }
    return await response.json();
}

export async function getRequestOrError<T>(endpoint: string): Promise<SimpleResult<T>> {
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
        return 'ERROR';
    }
    return await response.json();
}

export async function createRequest<T>(endpoint: string, data: T): Promise<void> {
    const response = await fetchWithRetry(
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
            throw new Error(`Failed to create - server status: ${response.status}, message: ${message}`);
        }
        throw new Error(`Failed to create - server status: ${response.status}`);
    }
    return;
}

export async function updateRequest<T extends { id: string }>(endpoint: string, data: T): Promise<void> {
    const {id, ...rest} = data;
    const response = await fetchWithRetry(
        `${BASE_URL}${endpoint}/${id}`,
        {
            method: 'post',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(rest),
        },
    );

    if (!response.ok) {
        throw new Error(`Failed to update - server status: ${response.status}`);
    }
    return;
}

export async function deleteRequest(endpoint: string, id: string): Promise<void> {
    const response = await fetchWithRetry(
        `${BASE_URL}${endpoint}/${id}`,
        {
            method: 'delete',
        },
    );

    if (!response.ok) {
        throw new Error(`Failed to delete - server status: ${response.status}`);
    }
    return;
}

async function fetchWithRetry(url: RequestInfo, options: RequestInit, attempt = 1): Promise<Response> {
    return fetch(url, options)
        .then((response) => {
            if (response.ok) {
                return response;
            }
            if (response.status === 500) {
                if (attempt > 5) {
                    return response;
                } else {
                    console.log(`Fetch failed (status 500), url: ${url}, current attempt: ${attempt}`);
                    return fetchWithRetry(url, options, attempt + 1);
                }
            } else {
                throw new Error(`Request failed with status ${response.status} after ${attempt} attempts, url: ${url}`);
            }
        })
        .catch((e) => {
            if (attempt > 5) {
                throw new Error(e);
            } else {
                console.log(`Caught exception during fetch, url: ${url}, current attempt: ${attempt}, error: ${e}`);
                return fetchWithRetry(url, options, attempt + 1);
            }
        });
}
