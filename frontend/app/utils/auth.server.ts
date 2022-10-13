import {getSession} from "./sessions.server";

export const getSessionData = async (
    request: Request,
) => {
    const session = await getSession(request.headers.get("cookie"));
    return {
        idToken: session.get("idToken") as string | undefined,
        session,
    };
};
