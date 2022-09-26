import {getSessionData} from "~/utils/auth.server";
import jwt_decode from "jwt-decode";
import {createCookieSessionStorage, redirect} from "@remix-run/node";


// Learn more about cookies at MDN https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies
// With Firebase hosting your cookie must be named __session https://firebase.google.com/docs/hosting/manage-cache#using_cookies
const {getSession, commitSession, destroySession} =
    createCookieSessionStorage({
        cookie: {
            name: "__session",
            secrets: [process.env.COOKIE_SECRET!],
            sameSite: "lax",
            httpOnly: true,
            secure: true,
            path: "/",
            // Set session expiration to 5 days
            maxAge: 60 * 60 * 24 * 5,
        },
    });

export {getSession, commitSession, destroySession};

export async function requireUserId(
    request: Request,
    // redirectTo: string = new URL(request.url).pathname
) {
    const {idToken} = await getSessionData(request);
    if (!idToken) {
        // const searchParams = new URLSearchParams([["redirectTo", redirectTo]]);
        throw redirect('/');
    }

    const {user_id, name} = jwt_decode(idToken) as { user_id: string, name: string }
    return {userId: user_id, name: name};
}